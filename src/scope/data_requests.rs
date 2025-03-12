/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://getnlab.com
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nLab API
 *
 **************************************************************************************************/

use std::collections::VecDeque;
use std::error::Error;
use std::sync::{Arc, mpsc, RwLock};
use std::sync::mpsc::{Receiver, Sender};

use log::{trace, debug};

use super::AnalogInput;
use super::Command;
use super::commands::ScopeCommand;
use super::Nlab;
use super::Trigger;

/// Voltage information from all open channels at a given time
#[derive(Debug, Default, Clone)]
pub struct Sample {
    pub time_since_start: f64,
    pub data: [Option<f64>; Sample::num_channels() as usize],
}

impl Sample {
    pub const fn num_channels() -> u32 { 4 }

    pub fn clear(&mut self) {
        self.data = [None; Sample::num_channels() as usize];
    }
}

#[derive(Debug)]
pub(crate) struct DataRequest {
    pub channels: [AnalogInput; 4],
    pub sample_rate_hz: f64,
    pub remaining_samples: Arc<RwLock<u32>>,
    pub trigger: Trigger,
    pub sender: Sender<Sample>,
    pub stop_recv: Receiver<()>,

    data_collator: Arc<RwLock<[VecDeque<u16>; 4]>>,
}

/// Handle to an ongoing data sweep, holds received data from nLab
#[derive(Debug)]
pub struct SweepHandle {
    pub receiver: Receiver<Sample>,
    samples_remaining: Arc<RwLock<u32>>,
    stop_send: Sender<()>,
}

impl Nlab {
    pub fn request(&self, sample_rate_hz: f64, number_of_samples: u32, trigger: Option<Trigger>) -> SweepHandle {
        let (tx, rx) = mpsc::channel::<Sample>();
        let (stop_send, stop_recv) = mpsc::channel::<()>();

        let remaining_samples = Arc::new(RwLock::new(number_of_samples));
        let command = Command::RequestData(DataRequest {
            channels: [self.ch1, self.ch2, self.ch3, self.ch4],
            sample_rate_hz,
            remaining_samples: remaining_samples.clone(),
            trigger: trigger.unwrap_or_default(),
            sender: tx,
            stop_recv,
            data_collator: Default::default(),
        });

        if self.command_tx.send(command).is_err() {
            *remaining_samples.write().unwrap() = 0;
        }

        SweepHandle {
            receiver: rx,
            samples_remaining: remaining_samples,
            stop_send,
        }
    }
}

impl SweepHandle {
    pub fn remaining_samples(&self) -> u32 {
        *self.samples_remaining.read().unwrap()
    }

    pub fn stop(&self) {
        self.stop_send.send(()).ok();
    }
}

impl ScopeCommand for DataRequest {
    fn fill_tx_buffer_legacy(&self, usb_buf: &mut [u8; 65]) -> Result<(), Box<dyn Error>> {
        usb_buf[1] = 0x08;

        let num_channels_on = self.channels.iter().filter(|&ch| ch.is_on).count();


        let samples_between_records: u32 = match num_channels_on {
            0 => { return Err("No scope channels are on".into()); }
            1 => { (4_000_000.0 / self.sample_rate_hz) as u32 }
            2 => { (2_000_000.0 / self.sample_rate_hz) as u32 }
            3 | 4 => { (1_000_000.0 / self.sample_rate_hz) as u32 }
            _ => { return Err("Unexpected number of channels are on".into()); }
        };

        let total_samples = *self.remaining_samples.read().unwrap();
        if samples_between_records < 250 && total_samples * num_channels_on as u32 > 3200 {
            return Err("Data not recordable".into());
        }


        usb_buf[3..=6].copy_from_slice(&samples_between_records.to_le_bytes());
        usb_buf[7..=10].copy_from_slice(&total_samples.to_le_bytes());
        trace!("Requesting {} samples with {} samples between records", total_samples, samples_between_records);

        if self.trigger.is_enabled {
            usb_buf[11] = self.trigger.source_channel as u8 | (self.trigger.trigger_type.value() << 2);

            if !(0..4usize).contains(&self.trigger.source_channel) {
                return Err("Invalid trigger channel".into());
            }
            let trigger_channel = self.channels[self.trigger.source_channel];
            let trigger_level = trigger_channel.measurement_from_voltage(self.trigger.trigger_level);
            if !(105..3990).contains(&trigger_level) {
                return Err("Trigger level is outside operating range of the channel".into());
            }
            let trigger_level = trigger_level as u16;
            usb_buf[11] |= ((trigger_level & 0x000F) << 4) as u8;
            usb_buf[12] = ((trigger_level & 0x0FF0) >> 4) as u8;

            let trigger_delay: u16 = match num_channels_on {
                1 => { 4 * self.trigger.trigger_delay_us / samples_between_records }
                2 => { 2 * self.trigger.trigger_delay_us / samples_between_records }
                3..=4 => { self.trigger.trigger_delay_us / samples_between_records }
                _ => { 1 }
            } as u16;
            usb_buf[13..=14].copy_from_slice(&trigger_delay.to_le_bytes());
        } else {
            usb_buf[11..=14].fill(0);
        }


        for (i, ch) in self.channels.iter().enumerate() {
            if ch.is_on {
                usb_buf[15 + i] = ch.gain_cmd();
                usb_buf[19 + i] = ch.offset_cmd();
            } else {
                usb_buf[15 + i] = 0xFF;
            }
        }


        Ok(())
    }

    fn fill_tx_buffer(&self, usb_buf: &mut [u8; 64]) -> Result<(), Box<dyn Error>> {
        let samples_between_records: u32 = (2_000_000.0 / self.sample_rate_hz) as u32;

        let total_samples = *self.remaining_samples.read().unwrap();
        debug!("Requesting {} samples with {} samples between records", total_samples, samples_between_records);
        if samples_between_records < 25 && total_samples > 2400 {
            return Err("Data not recordable".into());
        }

        usb_buf[2..6].copy_from_slice(&samples_between_records.to_le_bytes());
        usb_buf[6..10].copy_from_slice(&total_samples.to_le_bytes());

        // Fill bytes 10-13 with the channel gains (or 0xFF to indicate off)
        for (i, ch) in self.channels.iter().enumerate() {
            if ch.is_on {
                usb_buf[10 + i] = ch.gain_cmd();
                // usb_buf[14 + i] = ch.offset_setting;
            } else {
                usb_buf[10 + i] = 0xFF;
            }
        }

        // Fill trigger bytes
        // 14: trigger type
        // 15: source channel
        // 16-17: trigger level
        // 18-21: trigger delay

        if self.trigger.is_enabled {
            if !(0..4usize).contains(&self.trigger.source_channel) {
                return Err("Invalid trigger channel".into());
            }

            usb_buf[14] = self.trigger.trigger_type.value();
            usb_buf[15] = self.trigger.source_channel as u8;

            let trigger_channel = self.channels[self.trigger.source_channel];
            let trigger_level = trigger_channel.measurement_from_voltage(self.trigger.trigger_level);
            if !(5..4090).contains(&trigger_level) {
                return Err("Trigger level is outside operating range of the channel".into());
            }
            let trigger_level = trigger_level as u16;

            usb_buf[16..=17].copy_from_slice(&trigger_level.to_le_bytes());

            let trigger_delay = 2 * self.trigger.trigger_delay_us / samples_between_records;
            debug!("Trigger Delay: {:?}", trigger_delay);
            usb_buf[18..=21].copy_from_slice(&trigger_delay.to_le_bytes());
        } else {
            usb_buf[14..=21].fill(0);
        }

        Ok(())
    }


    fn handle_rx_legacy(&self, usb_buf: &[u8; 64]) {
        let number_received_samples = usb_buf[3] as u32;

        {
            let mut remaining_samples = self.remaining_samples.write().unwrap();
            *remaining_samples -= number_received_samples;
            trace!("Received {} samples, {} samples remaining", number_received_samples, remaining_samples);
        }


        let mut total_parsed_readings: usize = 0;

        for _ in 0..number_received_samples {
            let mut sample = Sample {
                time_since_start: 0.0,
                data: [None; 4],
            };

            for (i, ch) in self.channels.iter().enumerate() {
                if ch.is_on {
                    let byte = 4 + total_parsed_readings / 2 * 3;

                    let adc_data = match total_parsed_readings & 1 {
                        0 => (usb_buf[byte] as u16) | (((usb_buf[byte + 1] & 0xF) as u16) << 8),
                        1 => (usb_buf[byte + 1] as u16 >> 4) | ((usb_buf[byte + 2] as u16) << 4),
                        _ => panic!("Unexpected behavior of odd/even bitmask")
                    };

                    trace!("Ch{}: ADCData: {} Vi: {}", i+1, adc_data, ch.voltage_from_measurement(adc_data));
                    sample.data[i] = Some(ch.voltage_from_measurement(adc_data));
                    total_parsed_readings += 1;
                }
            }

            self.sender.send(sample).unwrap();
        }
    }
    fn handle_rx(&self, _usb_buf: &[u8; 64]) {}

    fn is_finished(&self) -> bool {
        *self.remaining_samples.read().unwrap() == 0
    }
}


impl DataRequest {
    pub(crate) fn handle_incoming_data(&self, usb_buf: &[u8; 64], channel: usize) {
        let num_received = usb_buf[1] as usize;
        let mut num_parsed: usize = 0;
        while num_parsed < num_received {
            let byte: usize = 4 + num_parsed / 2 * 3;

            let adc_data = match num_parsed % 2 {
                0 => (usb_buf[byte] as u16) | (((usb_buf[byte + 1] & 0xF) as u16) << 8),
                1 => (usb_buf[byte + 1] as u16 >> 4) | ((usb_buf[byte + 2] as u16) << 4),
                _ => panic!("Unexpected behavior of odd/even bitmask")
            };
            self.data_collator.write().unwrap()[channel].push_back(adc_data);
            num_parsed += 1;
        }
    }

    pub(crate) fn collate_results(&self) {
        let data_collator = &mut *self.data_collator.write().unwrap();

        // Find the number of samples received for all channels that are on using filter and map
        let received_samples = data_collator
            .iter()
            .enumerate()
            .filter(|&(ch, _)| self.channels[ch].is_on)
            .map(|(_, collator_channel)| collator_channel.len())
            .collect::<Vec<usize>>();

        if let Some(&complete_samples) = received_samples.iter().min() {
            let mut samples_to_pop = complete_samples;
            while samples_to_pop > 0 {
                let mut sample = Sample {
                    time_since_start: 0.0,
                    data: [None; 4],
                };

                for (ch, input_buffer) in data_collator.iter_mut().enumerate() {
                    let channel = &self.channels[ch];

                    if channel.is_on {
                        let data = input_buffer.pop_front().unwrap();
                        sample.data[ch] = Some(channel.voltage_from_measurement(data));
                    }
                }
                self.sender.send(sample).unwrap();
                samples_to_pop -= 1;
            }


            if complete_samples > 0 {
                let mut remaining_samples = self.remaining_samples.write().unwrap();
                *remaining_samples -= complete_samples as u32;
                trace!("Received {} samples, {} samples remaining", complete_samples, remaining_samples);
            }
        }
    }
}
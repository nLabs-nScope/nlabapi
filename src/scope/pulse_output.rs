/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

use std::error::Error;
use std::sync::{Arc, mpsc, RwLock};
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use crate::Nscope;
use crate::scope::commands::{Command, ScopeCommand};
use crate::scope::NscopeState;

#[derive(Debug, Copy, Clone)]
pub enum PulsePreScale {
    One,
    Eight,
    SixtyFour,
    TwoFiftySix,
}

impl PulsePreScale {
    fn value(&self) -> u64 {
        match *self {
            PulsePreScale::One => 1,
            PulsePreScale::Eight => 8,
            PulsePreScale::SixtyFour => 64,
            PulsePreScale::TwoFiftySix => 256,
        }
    }
    fn register(&self) -> u8 {
        match *self {
            PulsePreScale::One => 0,
            PulsePreScale::Eight => 1,
            PulsePreScale::SixtyFour => 2,
            PulsePreScale::TwoFiftySix => 3,
        }
    }
}


/// Interface to a pulse output
#[derive(Debug, Copy, Clone)]
pub struct PulseOutput {
    pub is_on: bool,
    pub frequency: f64,
    pub duty: f64,
}

impl Default for PulseOutput {
    fn default() -> Self {
        PulseOutput {
            is_on: false,
            frequency: 1.0,
            duty: 0.5,
        }
    }
}

impl PulseOutput {
    fn period(&self) -> Duration {
        let period = 1.0 / self.frequency;
        Duration::from_secs_f64(period)
    }

    fn pulse_width(&self) -> Duration {
        let period = self.period();
        period.mul_f64(self.duty)
    }
}

impl Nscope {
    pub fn get_px(&self, channel: usize) -> PulseOutput {
        let state = self.state.read().unwrap();
        state.pulse_output[channel]
    }

    pub(crate) fn set_px(&self, channel: usize, px: PulseOutput) -> Receiver<PulseOutput> {
        // Create a method for the backend to communicate back to us what we want
        let (tx, rx) = mpsc::channel::<PulseOutput>();

        // Create the command to set an analog output
        let command = Command::SetPulseOutput(PxRequest {
            channel,
            px,
            sender: tx,
        });

        // Send the command to the backend
        self.command_tx.send(command).unwrap();
        rx
    }

    pub fn set_px_on(&self, channel: usize, on: bool) -> PulseOutput {
        // Get the current state of the analog output
        let mut requested_px = self.get_px(channel);
        requested_px.is_on = on;

        let rx = self.set_px(channel, requested_px);

        // Wait for the backend to receive a response and return the result
        rx.recv().unwrap()
    }

    pub fn set_px_frequency_hz(&self, channel: usize, freq: f64) -> PulseOutput {
        // Get the current state of the analog output
        let mut requested_px = self.get_px(channel);
        requested_px.frequency = freq;

        let rx = self.set_px(channel, requested_px);

        // Wait for the backend to receive a response and return the result
        rx.recv().unwrap()
    }

    pub fn set_px_duty(&self, channel: usize, duty: f64) -> PulseOutput {
        // Get the current state of the analog output
        let mut requested_px = self.get_px(channel);
        requested_px.duty = duty;

        let rx = self.set_px(channel, requested_px);

        // Wait for the backend to receive a response and return the result
        rx.recv().unwrap()
    }
}


fn get_registers(pulse_output: &PulseOutput) -> Result<(u8, u32, u32), Box<dyn Error>> {

    // The period and duty registers are an integeter number of 16 MHz clock cycles
    let period = (pulse_output.period().as_nanos() * 16 / 1000) as u64;
    let duty = (pulse_output.pulse_width().as_nanos() * 16 / 1000) as u64;

    let prescale = if period < 4u64 {
        return Err("Desired pulse length is too short".into());
    } else if period <= u16::MAX as u64 {
        PulsePreScale::One
    } else if period <= u16::MAX as u64 * PulsePreScale::Eight.value() {
        PulsePreScale::Eight
    } else if period <= u16::MAX as u64 * PulsePreScale::SixtyFour.value() {
        PulsePreScale::SixtyFour
    } else if period <= u16::MAX as u64 * PulsePreScale::TwoFiftySix.value() {
        PulsePreScale::TwoFiftySix
    } else {
        return Err("Desired pulse length is too long".into());
    };

    let period_register = (period / (prescale.value())) as u32;
    let duty_register = (duty / (prescale.value())) as u32;

    Ok((prescale.register(), period_register, duty_register))
}

#[derive(Debug)]
pub(super) struct PxRequest {
    channel: usize,
    px: PulseOutput,
    sender: Sender<PulseOutput>,
}

impl ScopeCommand for PxRequest {
    fn fill_tx_buffer(&self, usb_buf: &mut [u8; 65]) -> Result<(), Box<dyn Error>> {
        usb_buf[1] = 0x01;

        let i_ch = 3 + 10 * self.channel;
        let (prescale, period, duty) = get_registers(&self.px)?;

        if self.px.is_on {
            usb_buf[i_ch] = 0x80 | prescale;
            usb_buf[i_ch + 1..=i_ch + 4].copy_from_slice(&period.to_le_bytes());
            usb_buf[i_ch + 5..=i_ch + 8].copy_from_slice(&duty.to_le_bytes());
        } else {
            usb_buf[i_ch] = 0xFF;
        }

        Ok(())
    }

    fn handle_rx(self, _usb_buf: &[u8; 64], scope_state: &Arc<RwLock<NscopeState>>) -> Option<Self> {
        let mut state = scope_state.write().unwrap();
        state.pulse_output[self.channel] = self.px;
        self.sender.send(self.px).unwrap();
        None
    }
}

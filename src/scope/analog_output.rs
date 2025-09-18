/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://getnlab.com
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nLab API
 *
 **************************************************************************************************/

use std::error::Error;
use std::str::FromStr;
use std::sync::{mpsc, RwLock};
use std::sync::mpsc::Sender;
#[cfg(feature = "python_support")] use pyo3::pyclass;

use crate::scope::commands::ScopeCommand;

use super::commands::Command;

/// Possible analog output signal types
#[derive(Debug, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "python_support", pyclass(eq, eq_int))]
pub enum AnalogWaveType {
    Sine = 0,
    Triangle = 1,
}

impl FromStr for AnalogWaveType {
    type Err = ();
    fn from_str(input: &str) -> Result<AnalogWaveType, Self::Err> {
        match input {
            "Sine" => Ok(AnalogWaveType::Sine),
            "Triangle" => Ok(AnalogWaveType::Triangle),
            _ => Err(()),
        }
    }
}

/// Possible analog output polarities
#[derive(Debug, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "python_support", pyclass(eq, eq_int))]
pub enum AnalogSignalPolarity {
    Unipolar = 0,
    Bipolar = 1,
}

impl FromStr for AnalogSignalPolarity {
    type Err = ();
    fn from_str(input: &str) -> Result<AnalogSignalPolarity, Self::Err> {
        match input {
            "Unipolar" => Ok(AnalogSignalPolarity::Unipolar),
            "Bipolar" => Ok(AnalogSignalPolarity::Bipolar),
            _ => Err(()),
        }
    }
}

/// Interface to an analog output
#[derive(Clone, Copy, Debug)]
struct AnalogOutputState {
    is_on: bool,
    frequency: f64,
    amplitude: f64,
    wave_type: AnalogWaveType,
    polarity: AnalogSignalPolarity,
}

/// Interface to an analog output channel
#[derive(Debug)]
pub struct AnalogOutput {
    pub channel: usize,
    command_tx: Sender<Command>,
    state: RwLock<AnalogOutputState>,
}

impl AnalogOutput {
    pub(super) fn create(cmd_tx: Sender<Command>, ax_channel: usize) -> Self {
        let default_state = AnalogOutputState {
            is_on: false,
            frequency: 1.0,
            amplitude: 1.0,
            wave_type: AnalogWaveType::Sine,
            polarity: AnalogSignalPolarity::Unipolar,
        };

        let ax = AnalogOutput {
            command_tx: cmd_tx,
            channel: ax_channel,
            state: RwLock::new(default_state),
        };

        ax.set(default_state);
        ax
    }

    fn set(&self, ax_state: AnalogOutputState) {
        // Create a method for the backend to communicate back to us what we want
        let (tx, rx) = mpsc::channel::<AnalogOutputState>();

        // Create the command to set an analog output
        let command = Command::SetAnalogOutput(AxRequest {
            channel: self.channel,
            ax_state,
            sender: tx,

        });

        // Send the command to the backend
        if self.command_tx.send(command).is_ok() {

            // Wait for the response from the backend
            if let Ok(response_state) = rx.recv() {
                // Write the response state
                *self.state.write().unwrap() = response_state;
            }
        }
    }

    pub fn is_on(&self) -> bool {
        self.state.read().unwrap().is_on
    }
    pub fn frequency(&self) -> f64 {
        self.state.read().unwrap().frequency
    }
    pub fn amplitude(&self) -> f64 {
        self.state.read().unwrap().amplitude
    }
    pub fn wave_type(&self) -> AnalogWaveType {
        self.state.read().unwrap().wave_type
    }
    pub fn polarity(&self) -> AnalogSignalPolarity {
        self.state.read().unwrap().polarity
    }


    pub fn turn_on(&self) {
        let mut state = *self.state.read().unwrap();
        state.is_on = true;
        self.set(state)
    }
    pub fn turn_off(&self) {
        let mut state = *self.state.read().unwrap();
        state.is_on = false;
        self.set(state)
    }

    pub fn set_frequency(&self, desired_hz: f64) {
        let mut state = *self.state.read().unwrap();
        state.frequency = desired_hz;
        self.set(state)
    }

    pub fn set_amplitude(&self, desired_volts: f64) {
        let mut state = *self.state.read().unwrap();
        state.amplitude = desired_volts;
        self.set(state)
    }

    pub fn set_wave_type(&self, wave_type: AnalogWaveType) {
        let mut state = *self.state.read().unwrap();
        state.wave_type = wave_type;
        self.set(state)
    }

    pub fn set_polarity(&self, polarity: AnalogSignalPolarity) {
        let mut state = *self.state.read().unwrap();
        state.polarity = polarity;
        self.set(state)
    }
}


#[derive(Debug)]
pub(crate) struct AxRequest {
    channel: usize,
    ax_state: AnalogOutputState,
    sender: Sender<AnalogOutputState>,
}

impl ScopeCommand for AxRequest {
    fn fill_tx_buffer_legacy(&self, usb_buf: &mut [u8; 65]) -> Result<(), Box<dyn Error>> {
        usb_buf[1] = 0x02;

        let i_ch = 3 + 10 * self.channel;
        if self.ax_state.is_on {
            usb_buf[i_ch] = self.ax_state.wave_type as u8;
            usb_buf[i_ch] |= 0x80;

            let scaled_frequency = self.ax_state.frequency * 2.0_f64.powi(28) / 4000000.0;
            let freq_register: u32 = scaled_frequency as u32;

            usb_buf[i_ch + 1] = (freq_register & 0x00FF) as u8;
            usb_buf[i_ch + 2] = ((freq_register & 0x3F00) >> 8) as u8;
            usb_buf[i_ch + 3] = ((freq_register >> 14) & 0x00FF) as u8;
            usb_buf[i_ch + 4] = (((freq_register >> 14) & 0x3F00) >> 8) as u8;

            if self.ax_state.amplitude < 0.0 {
                usb_buf[i_ch] |= 0x2;
            }
            let rf = 49900.0;
            let vin = 0.6;
            let rm = 75.0;
            let rv = 100000.0 / 257.0;

            let gain: u8 = match self.ax_state.polarity {
                AnalogSignalPolarity::Unipolar => ((vin * rf / self.ax_state.amplitude.abs() - rm) / rv) as u8,
                AnalogSignalPolarity::Bipolar => {
                    ((vin * rf / 2.0 / self.ax_state.amplitude.abs() - rm) / rv) as u8
                }
            };

            let offset: u8 = ((rm + rv * (gain as f64)) / (rm + rv * (gain as f64) + rf)
                * self.ax_state.amplitude.abs()
                * 255.0
                / 3.05) as u8;

            usb_buf[i_ch + 5] = gain;
            usb_buf[i_ch + 6] = offset;
        } else {
            usb_buf[i_ch] = 0xFF;
        }
        Ok(())
    }

    fn fill_tx_buffer(&self, usb_buf: &mut [u8; 64]) -> Result<(), Box<dyn Error>> {
        // Set the channel of interest
        usb_buf[3] = 0x1 << self.channel;

        let idx_start = 4 + 12 * self.channel;

        usb_buf[idx_start] = self.ax_state.is_on as u8;
        usb_buf[idx_start + 1..=idx_start + 4].copy_from_slice(
            &(self.ax_state.frequency as f32).to_le_bytes());
        usb_buf[idx_start + 5..=idx_start + 8].copy_from_slice(
            &(self.ax_state.amplitude as f32).to_le_bytes());
        usb_buf[idx_start + 9] = self.ax_state.wave_type as u8;
        usb_buf[idx_start + 10] = self.ax_state.polarity as u8;

        Ok(())
    }

    fn handle_rx_legacy(&self, _usb_buf: &[u8; 64]) {
        self.sender.send(self.ax_state).unwrap();
    }

    fn handle_rx(&self, _usb_buf: &[u8; 64]) {
        self.sender.send(self.ax_state).unwrap();
    }

    fn is_finished(&self) -> bool {
        true
    }
}
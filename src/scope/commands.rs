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

use log::debug;

use super::analog_output::AxRequest;
use super::data_requests::{DataRequest, StopRequest};
use super::pulse_output::PxRequest;

pub(super) const NULL_REQ: [u8; 2] = [0, 0xFF];

pub(super) trait ScopeCommand {
    fn fill_tx_buffer(&self, usb_buf: &mut [u8; 65]) -> Result<(), Box<dyn Error>>;
    fn handle_rx(&self, usb_buf: &[u8; 64]);
    fn is_finished(&self) -> bool;
}

// Build out featureset
// PWM_DUTY_REQUEST = 0x00, -- not for 1.0
// FINITE_DATA_REQUEST = 0x03, -- not for 1.0
// CONTINUOUS_DATA_REQUEST = 0x04 -- not for 1.0
// SCOPE_ROLL_REQUEST = 0x09 -- not for 1.0
// RESET_TO_BOOTLOADER = 0x10 -- not for 1.0


#[derive(Debug)]
pub(crate) enum Command {
    Quit,
    Initialize(bool),
    SetAnalogOutput(AxRequest),
    SetPulseOutput(PxRequest),
    RequestData(DataRequest),
    StopData(StopRequest),
}

impl Command {
    pub(super) fn fill_tx_buffer(&mut self, usb_buf: &mut [u8; 65]) -> Result<(), Box<dyn Error>> {
        debug!("Processed command: {:?}", self);
        match self {
            Command::Quit => { Ok(()) }
            Command::Initialize(power_on) => {
                if *power_on {
                    usb_buf[1] = 0x07;
                } else {
                    usb_buf[1] = 0x06;
                }
                Ok(())
            }
            Command::SetAnalogOutput(cmd) => { cmd.fill_tx_buffer(usb_buf) }
            Command::SetPulseOutput(cmd) => { cmd.fill_tx_buffer(usb_buf) }
            Command::RequestData(cmd) => { cmd.fill_tx_buffer(usb_buf) }
            Command::StopData(cmd) => { cmd.fill_tx_buffer(usb_buf) }
        }
    }

    pub(super) fn handle_rx(&self, buffer: &[u8; 64]) {
        match self {
            Command::Quit => {}
            Command::Initialize(_) =>  {}
            Command::SetAnalogOutput(cmd) => { cmd.handle_rx(buffer) }
            Command::SetPulseOutput(cmd) => { cmd.handle_rx(buffer) }
            Command::RequestData(cmd) => { cmd.handle_rx(buffer) }
            Command::StopData(cmd) => { cmd.handle_rx(buffer) }
        }
    }

    pub(super) fn is_finished(&self) -> bool {
        match self {
            Command::Quit => { true }
            Command::Initialize(_) => { true }
            Command::SetAnalogOutput(cmd) => { cmd.is_finished() }
            Command::SetPulseOutput(cmd) => { cmd.is_finished() }
            Command::RequestData(cmd) => { cmd.is_finished() }
            Command::StopData(cmd) => { cmd.is_finished() }
        }
    }
}

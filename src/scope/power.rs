/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://getnlab.com
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nLab API
 *
 **************************************************************************************************/

use std::io;
use pyo3::{pyclass, pymethods};
use super::Nlab;

/// Information about the power supply status of nLab
#[derive(Debug, Copy, Clone)]
#[pyclass]
pub struct PowerStatus {
    #[pyo3(get)]
    pub state: PowerState,
    #[pyo3(get)]
    pub usage: f64,
}

impl Default for PowerStatus {
    fn default() -> Self {
        PowerStatus {
            state: PowerState::Unknown,
            usage: 0.0,
        }
    }
}

#[pymethods]
impl PowerStatus {
    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}

/// Possible states of the nLab power supply
#[derive(Debug, PartialEq, Copy, Clone)]
#[pyclass(eq, eq_int)]
pub enum PowerState {
    PowerOff,
    PowerOn,
    Shorted,
    Overcurrent,
    Startup,
    Unknown,
}

impl From<u8> for PowerState {
    fn from(orig: u8) -> Self {
        match orig {
            0 => PowerState::PowerOff,
            1 => PowerState::PowerOn,
            2 => PowerState::Shorted,
            3 => PowerState::Overcurrent,
            4 => PowerState::Startup,
            _ => PowerState::Unknown,
        }
    }
}

impl Nlab {
    pub fn power_status(&self) -> Result<PowerStatus, io::Error> {
        if !self.is_connected() {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "nLab connection aborted",
            ));
        }
        Ok(*self.power_status.read().unwrap())
    }
}

use pyo3::exceptions::*;
use pyo3::prelude::*;

use crate::{AnalogSignalPolarity, AnalogWaveType, python};


#[pymethods]
impl python::Nlab {
    fn ax_is_on(&self, ch: i64) -> PyResult<bool> {
        let scope: &crate::Nlab = &self.0;

        let ax = match ch {
            1 => &scope.a1,
            2 => &scope.a2,
            _ => return Err(PyValueError::new_err(format!("Invalid channel number: {ch}")))
        };

        Ok(ax.is_on())
    }

    fn ax_frequency(&self, ch: i64) -> PyResult<f64> {
        let scope: &crate::Nlab = &self.0;

        let ax = match ch {
            1 => &scope.a1,
            2 => &scope.a2,
            _ => return Err(PyValueError::new_err(format!("Invalid channel number: {ch}")))
        };

        Ok(ax.frequency())
    }

    fn ax_amplitude(&self, ch: i64) -> PyResult<f64> {
        let scope: &crate::Nlab = &self.0;

        let ax = match ch {
            1 => &scope.a1,
            2 => &scope.a2,
            _ => return Err(PyValueError::new_err(format!("Invalid channel number: {ch}")))
        };

        Ok(ax.amplitude())
    }

    fn ax_wave_type(&self, ch: i64) -> PyResult<AnalogWaveType> {
        let scope: &crate::Nlab = &self.0;

        let ax = match ch {
            1 => &scope.a1,
            2 => &scope.a2,
            _ => return Err(PyValueError::new_err(format!("Invalid channel number: {ch}")))
        };

        Ok(ax.wave_type())
    }

    fn ax_polarity(&self, ch: i64) -> PyResult<AnalogSignalPolarity> {
        let scope: &crate::Nlab = &self.0;

        let ax = match ch {
            1 => &scope.a1,
            2 => &scope.a2,
            _ => return Err(PyValueError::new_err(format!("Invalid channel number: {ch}")))
        };

        Ok(ax.polarity())
    }

    fn ax_turn_on(&self, ch: i64) -> PyResult<()> {
        let scope: &crate::Nlab = &self.0;

        let ax = match ch {
            1 => &scope.a1,
            2 => &scope.a2,
            _ => return Err(PyValueError::new_err(format!("Invalid channel number: {ch}")))
        };

        ax.turn_on();
        Ok(())
    }

    fn ax_turn_off(&self, ch: i64) -> PyResult<()> {
        let scope: &crate::Nlab = &self.0;

        let ax = match ch {
            1 => &scope.a1,
            2 => &scope.a2,
            _ => return Err(PyValueError::new_err(format!("Invalid channel number: {ch}")))
        };

        ax.turn_off();
        Ok(())
    }

    fn ax_set_frequency(&self, ch: i64, desired_hz: f64) -> PyResult<()> {
        let scope: &crate::Nlab = &self.0;

        let ax = match ch {
            1 => &scope.a1,
            2 => &scope.a2,
            _ => return Err(PyValueError::new_err(format!("Invalid channel number: {ch}")))
        };

        ax.set_frequency(desired_hz);
        Ok(())
    }

    fn ax_set_amplitude(&self, ch: i64, desired_volts: f64) -> PyResult<()> {
        let scope: &crate::Nlab = &self.0;

        let ax = match ch {
            1 => &scope.a1,
            2 => &scope.a2,
            _ => return Err(PyValueError::new_err(format!("Invalid channel number: {ch}")))
        };

        ax.set_amplitude(desired_volts);
        Ok(())
    }

    fn ax_set_wave_type(&self, ch: i64, wave_type: AnalogWaveType) -> PyResult<()> {
        let scope: &crate::Nlab = &self.0;

        let ax = match ch {
            1 => &scope.a1,
            2 => &scope.a2,
            _ => return Err(PyValueError::new_err(format!("Invalid channel number: {ch}")))
        };

        ax.set_wave_type(wave_type);
        Ok(())
    }

    fn ax_set_polarity(&self, ch: i64, polarity: AnalogSignalPolarity) -> PyResult<()> {
        let scope: &crate::Nlab = &self.0;

        let ax = match ch {
            1 => &scope.a1,
            2 => &scope.a2,
            _ => return Err(PyValueError::new_err(format!("Invalid channel number: {ch}")))
        };

        ax.set_polarity(polarity);
        Ok(())
    }
}
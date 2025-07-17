use pyo3::exceptions::*;
use pyo3::prelude::*;

use crate::python;


#[pymethods]
impl python::Nlab {
    fn px_is_on(&self, ch: i64) -> PyResult<bool> {
        let scope: &crate::Nlab = &self.0;

        let px = match ch {
            1 => &scope.p1,
            2 => &scope.p2,
            _ => return Err(PyValueError::new_err(format!("Invalid channel number: {ch}")))
        };

        Ok(px.is_on())
    }

    fn px_frequency(&self, ch: i64) -> PyResult<f64> {
        let scope: &crate::Nlab = &self.0;

        let px = match ch {
            1 => &scope.p1,
            2 => &scope.p2,
            _ => return Err(PyValueError::new_err(format!("Invalid channel number: {ch}")))
        };

        Ok(px.frequency())
    }

    fn px_duty(&self, ch: i64) -> PyResult<f64> {
        let scope: &crate::Nlab = &self.0;

        let px = match ch {
            1 => &scope.p1,
            2 => &scope.p2,
            _ => return Err(PyValueError::new_err(format!("Invalid channel number: {ch}")))
        };

        Ok(px.duty())
    }

    fn px_period(&self, ch: i64) -> PyResult<f64> {
        let scope: &crate::Nlab = &self.0;

        let px = match ch {
            1 => &scope.p1,
            2 => &scope.p2,
            _ => return Err(PyValueError::new_err(format!("Invalid channel number: {ch}")))
        };

        Ok(px.period().as_secs_f64())
    }

    fn px_pulse_width(&self, ch: i64) -> PyResult<f64> {
        let scope: &crate::Nlab = &self.0;

        let px = match ch {
            1 => &scope.p1,
            2 => &scope.p2,
            _ => return Err(PyValueError::new_err(format!("Invalid channel number: {ch}")))
        };

        Ok(px.pulse_width().as_secs_f64())
    }

    fn px_turn_on(&self, ch: i64) -> PyResult<()> {
        let scope: &crate::Nlab = &self.0;

        let px = match ch {
            1 => &scope.p1,
            2 => &scope.p2,
            _ => return Err(PyValueError::new_err(format!("Invalid channel number: {ch}")))
        };

        px.turn_on();
        Ok(())
    }

    fn px_turn_off(&self, ch: i64) -> PyResult<()> {
        let scope: &crate::Nlab = &self.0;

        let px = match ch {
            1 => &scope.p1,
            2 => &scope.p2,
            _ => return Err(PyValueError::new_err(format!("Invalid channel number: {ch}")))
        };

        px.turn_off();
        Ok(())
    }

    fn px_set_frequency(&self, ch: i64, desired_hz: f64) -> PyResult<()> {
        let scope: &crate::Nlab = &self.0;

        let px = match ch {
            1 => &scope.p1,
            2 => &scope.p2,
            _ => return Err(PyValueError::new_err(format!("Invalid channel number: {ch}")))
        };

        px.set_frequency(desired_hz);
        Ok(())
    }

    fn px_set_duty(&self, ch: i64, desired_percentage: f64) -> PyResult<()> {
        let scope: &crate::Nlab = &self.0;

        let px = match ch {
            1 => &scope.p1,
            2 => &scope.p2,
            _ => return Err(PyValueError::new_err(format!("Invalid channel number: {ch}")))
        };

        px.set_duty(desired_percentage);
        Ok(())
    }
}
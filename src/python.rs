mod bench;
mod scope;
mod analog_output;
mod pulse_output;
mod cli;

use pyo3::prelude::*;
use crate::{AnalogSignalPolarity, AnalogWaveType, PowerStatus, PowerState};
use cli::{Cli, Commands};
use clap::Parser;

#[pyclass]
struct LabBench;

#[pyclass]
struct Nlab(crate::Nlab);

#[pyfunction]
fn run_cli(_py: Python) -> PyResult<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let cli = Cli::parse_from(args);

    match &cli.command {
        Commands::Update(args) => { LabBench::update_all_nlabs(args.force_downgrade) }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn nlabapi(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<LabBench>()?;
    m.add_class::<Nlab>()?;
    m.add_class::<AnalogWaveType>()?;
    m.add_class::<AnalogSignalPolarity>()?;
    m.add_class::<PowerState>()?;
    m.add_class::<PowerStatus>()?;
    m.add_function(wrap_pyfunction!(run_cli, m)?)?;
    Ok(())
}
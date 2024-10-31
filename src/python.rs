mod bench;
mod scope;
mod analog_output;
mod pulse_output;
mod cli;

use std::thread;
use std::time::Duration;
use pyo3::prelude::*;
use crate::{AnalogSignalPolarity, AnalogWaveType, PowerStatus, PowerState, LabBench as NativeLabBench};
use cli::{Cli, Commands};
use clap::Parser;

#[pyfunction]
fn run_cli(_py: Python) -> PyResult<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let cli = Cli::parse_from(args);

    match &cli.command {
        Commands::Update => {
            if let Ok(mut bench) = NativeLabBench::new() {
                for nlab_link in bench.list() {
                    // Request DFU on any nLab that is available
                    if nlab_link.available {
                        if let Err(e) = nlab_link.request_dfu() {
                            println!("Failed to request DFU on an available nLab: {e}")
                        }
                    }
                }
                // Wait 500ms for the scope to detach and re-attach as DFU
                thread::sleep(Duration::from_millis(500));
                bench.refresh();

                for nlab_link in bench.list() {
                    if let Err(e) = nlab_link.update() {
                        println!("Encountered an error updating nLab: {e}")
                    }
                }
            }
        }
    }

    Ok(())
}

#[pyclass]
struct LabBench;

#[pyclass]
struct Nlab(crate::Nlab);

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
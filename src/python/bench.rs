use std::thread;
use std::time::Duration;
use pyo3::exceptions::*;
use pyo3::prelude::*;
use crate::{LabBench, python};

#[pymethods]
impl python::LabBench {
    #[staticmethod]
    fn open_first_available() -> PyResult<python::Nlab> {
        if let Ok(bench) = LabBench::new() {
            return match bench.open_first_available(true) {
                Ok(scope) => Ok(python::Nlab(scope)),
                Err(err) => Err(PyRuntimeError::new_err(err)),
            };
        }
        Err(PyRuntimeError::new_err("Cannot create LabBench"))
    }

    #[staticmethod]
    fn list_all_nlabs() {
        if let Ok(bench) = LabBench::new() {
            for nlab_link in bench.list() {
                println!("{nlab_link:?}");
            }
        } else {
            println!("Cannot create LabBench");
        }
    }

    #[staticmethod]
    fn count_connected_nlabs() -> usize {
        if let Ok(bench) = LabBench::new() {
            return bench.list().count();
        }
        0
    }

    #[staticmethod]
    pub(super) fn update_all_nlabs() -> PyResult<()> {
        if let Ok(mut bench) = LabBench::new() {
            for nlab_link in bench.list() {
                // Request DFU on any nLab that is available
                if nlab_link.available {
                    if let Err(e) = nlab_link.request_dfu() {
                        println!("Failed to request DFU on an available nLab: {e}");
                        return Err(PyRuntimeError::new_err(format!("{e}")));
                    }
                }
            }
            println!("Updating all connected nLabs...");
            // Wait 500ms for the scope to detach and re-attach as DFU
            thread::sleep(Duration::from_millis(500));
            bench.refresh();

            for nlab_link in bench.list() {
                if let Err(e) = nlab_link.update() {
                    println!("Encountered an error updating nLab: {e}");
                    return Err(PyRuntimeError::new_err(format!("{e}")));
                }
            }
            println!("Update complete!");
        }
        Ok(())
    }
}

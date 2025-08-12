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
                Err(err) => {
                    if err.to_string().contains("downgrade") {
                        let error_lines = [
                            "Device detected with newer firmware than the installed nlabapi.",
                            "To ensure compatibility, please update nlabapi by running:",
                            "    pip install --upgrade nlabapi",
                        ];
                        Err(PyRuntimeError::new_err(error_lines.join("\n")))
                    } else {
                        Err(PyRuntimeError::new_err(err))
                    }
                },
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
    #[pyo3(signature = (force_downgrade=false))]
    pub(super) fn update_all_nlabs(force_downgrade:bool) -> PyResult<()> {
        if let Ok(mut bench) = LabBench::new() {
            if bench.list().count() == 0 {
                return Err(PyRuntimeError::new_err("No nLab devices found."));
            }

            for nlab_link in bench.list() {
                if nlab_link.must_be_downgraded() {
                    let error_lines = [
                        "Device detected with newer firmware than the installed nlabapi.",
                        "To ensure compatibility, please update nlabapi by running:",
                        "    pip install --upgrade nlabapi",
                    ];
                    if force_downgrade {
                        println!("{}", error_lines.join("\n"));
                        println!("Forcing downgrade by request")
                    } else {
                        return Err(PyRuntimeError::new_err(error_lines.join("\n")));
                    }
                }
            }

            let mut device_update_count = 0;
            for nlab_link in bench.list() {
                if nlab_link.needs_update {
                    if let Err(e) = nlab_link.request_dfu() {
                        println!("Failed to request DFU on an available nLab: {e}");
                        return Err(PyRuntimeError::new_err(format!("{e}")));
                    } else {
                        device_update_count += 1;
                    }
                }
            }

            if device_update_count == 0 {
                println!("No firmware updates are needed for connected nLab devices.");
                return Ok(());
            }
            match device_update_count {
                1 => { println!("Updating connected nLab...") }
                _ => { println!("Updating {device_update_count} connected nLabs...") }
            }

            // Wait 500ms for the scope to detach and re-attach as DFU
            thread::sleep(Duration::from_millis(500));
            bench.refresh();

            for nlab_link in bench.list() {
                if nlab_link.in_dfu {
                    if let Err(e) = nlab_link.update() {
                        println!("Encountered an error updating nLab: {e}");
                        return Err(PyRuntimeError::new_err(format!("{e}")));
                    } else {
                        device_update_count -= 1;
                    }
                }
            }
            if device_update_count == 0 {
                println!("Update complete!");
            } else {
                match device_update_count {
                    1 => { println!("Failed to update {device_update_count} nLab") }
                    _ => { println!("Failed to update {device_update_count} nLabs") }
                }
            }
        }
        Ok(())
    }
}

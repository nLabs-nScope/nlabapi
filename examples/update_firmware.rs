/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://getnlab.com
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nLab API
 *
 **************************************************************************************************/

use nlabapi::LabBench;
use std::error::Error;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // Create a LabBench
    let mut bench = LabBench::new()?;

    // Print the bench to show a list of detected nLabs
    println!("{bench:?}");

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

    // Print the bench to show the refreshed list
    println!("{bench:?}");

    for nlab_link in bench.list() {
        if let Err(e) = nlab_link.update() {
            println!("Encountered an error updating nLab: {e}")
        }
    }

    Ok(())
}

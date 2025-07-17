/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://getnlab.com
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nLab API
 *
 **************************************************************************************************/

use nlabapi::{LabBench, Nlab};
use std::{thread, time};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // Create a LabBench
    let bench = LabBench::new()?;

    // Open all available nLab links
    let mut nlabs: Vec<Nlab> = bench.open_all_available();

    loop {
        thread::sleep(time::Duration::from_millis(500));

        nlabs.retain(|n| n.is_connected());

        for n in nlabs.iter() {
            match n.power_status() {
                Ok(status) => {
                    let state = format!("{:?}", status.state);
                    println!("{:>15}: {:.3} Watts", state, status.usage)
                },
                Err(error) => eprintln!("{error}"),
            }
        }

        if nlabs.is_empty() {
            return Err("Cannot find any nLabs".into())
        }
    }
}

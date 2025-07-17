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

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // Create a LabBench
    let bench = LabBench::new()?;

    // Print the bench to show a list of detected nLabs
    println!("Lab Bench: \n{bench:?}");

    println!("\nManual list of all detected nLabs:");
    // Or loop over all nLab links in the list and print them
    for nlab_link in bench.list() {
        println!("    {nlab_link:?}")
    }
    Ok(())
}

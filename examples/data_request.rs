/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://getnlab.com
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nLab API
 *
 **************************************************************************************************/

use nlabapi::{AnalogSignalPolarity, LabBench, Trigger, TriggerType};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // Create a LabBench
    let bench = LabBench::new()?;

    // Open the first available nLab
    let nlab = bench.open_first_available(true)?;

    nlab.a1.turn_on();

    let sweep_handle = nlab.request(100000.0, 3000, None);

    for sample in sweep_handle.receiver {
        println!("{:?}", sample.data);
    }

    nlab.a1.turn_off();
    
    
    let sweep_handle = nlab.request(100000.0, 3000, Some(Trigger{
        is_enabled: true,
        trigger_type: TriggerType::RisingEdge,
        source_channel: 0,
        trigger_level: 0.0,
        trigger_delay_us: 0,
    }));

    nlab.a1.set_polarity(AnalogSignalPolarity::Bipolar);
    nlab.a1.turn_on();
    for sample in sweep_handle.receiver {
        println!("{:?}", sample.data);
    }

    Ok(())
}
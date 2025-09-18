/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://getnlab.com
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nLab API
 *
 **************************************************************************************************/

//! This crate provides an interface to the [nLab](https://getnlab.com)
//!
//! # Usage
//!
//! This crate is [on crates.io](https://crates.io/crates/nlabapi) and can be
//! used by adding `nlab` to the dependencies in your project's `Cargo.toml`.
//!
//!
//! # Example
//!
//! ```rust,no_run
//! extern crate nlabapi;
//! use nlabapi::LabBench;
//!
//! fn main() {
//!     // Create a LabBench
//!     let bench = LabBench::new().expect("Cannot create LabBench");
//!
//!     // Print the bench to show a list of detected nLabs
//!     println!("{:?}", bench);
//!
//!     // Open an nLab
//!     let nlab = bench.open_first_available(true).expect("Cannot open nLab");
//!
//!     // Turn on analog output channel A1
//!     nlab.a1.turn_on();
//!
//!     // Trigger an auto-triggered sweep of 20 samples at 4.0 Hz sample rate
//!     let sweep_handle = nlab.request(4.0, 20, None);
//!
//!     // Loop through the received data, blocking on each sample until it arrives
//!     for sample in sweep_handle.receiver {
//!         // Print the sample data
//!         println!("{:?}", sample.data);
//!     }
//!
//!     // Turn off the analog output channel A1
//!     nlab.a1.turn_off();
//!
//! }
//! ```


mod lab_bench;
mod scope;
mod version;
mod firmware;
#[cfg(feature = "python_support")] mod python;

pub use lab_bench::LabBench;
pub use lab_bench::NlabLink;
pub use scope::Nlab;
pub use scope::power::*;
pub use scope::pulse_output::*;
pub use scope::analog_output::*;
pub use scope::analog_input::*;
pub use scope::data_requests::*;
pub use scope::trigger::*;
pub use version::version;

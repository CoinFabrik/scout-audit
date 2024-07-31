#![feature(internal_output_capture)]
extern crate lazy_static;
#[macro_use]
extern crate prettytable;

#[path = "../build_config/mod.rs"]
pub mod build_config;

pub mod detectors;
pub mod output;
pub mod scout;
pub mod server;
pub mod startup;
pub mod utils;

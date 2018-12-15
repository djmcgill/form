#![recursion_limit = "1024"]
extern crate getopts;
extern crate quote;
extern crate syn;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate failure;

use log::LevelFilter;

mod opts;
use crate::opts::FormOpts;
use failure::*;

mod util;
use crate::util::create_directory_structure;

fn main() {
    match run() {
        Ok(()) => println!("Completed successfully"),
        Err(error) => println!("Failed with:\n {}", error),
    }
}

fn run() -> Result<(), Error> {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .try_init()
        .context("could not initialise env_logger")?;

    trace!("logging initialised");
    let try_parsed_args =
        FormOpts::from_args().context("could not parse the command line arguments")?;
    // if None, we've already printed a help text and have nothing more to do
    if let Some(opts) = try_parsed_args {
        create_directory_structure(opts.output_dir, opts.input)?;
    }
    return Ok(());
}

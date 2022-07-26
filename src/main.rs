#![recursion_limit = "1024"]
extern crate getopts;
extern crate quote;
extern crate syn;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate failure;

use env_logger::Env;

mod opts;
use crate::opts::FormOpts;
use failure::*;

mod util;
use crate::util::create_directory_structure;

const NAME: &str = "form";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    if let Err(error) = run() {
        eprintln!("Failed with:\n {}", error);
    }
}

fn run() -> Result<(), Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .try_init()
        .context("could not initialise env_logger")?;

    trace!("logging initialised");

    let try_parsed_args =
        FormOpts::from_args().context("could not parse the command line arguments")?;
    // if None, we've already printed a help text or version and have nothing more to do
    if let Some(opts) = try_parsed_args {
        create_directory_structure(opts.output_dir, &opts.input)?;
        println!("Completed successfully");
    }
    Ok(())
}

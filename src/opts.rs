use getopts::Options;
use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use failure::*;

pub struct FormOpts {
    pub input: String,
    pub output_dir: String,
}

impl FormOpts {
    pub fn from_args() -> Result<Option<Self>, Error> {
        const SHORT_INPUT: &str = "i";
        const SHORT_OUTDIR: &str = "o";
        const SHORT_HELP: &str = "h";
        const SHORT_VERSION: &str = "v";

        let args: Vec<String> = env::args().collect();
        let program = args[0].clone();
        let mut opts = Options::new();
        opts.optopt(
            SHORT_INPUT,
            "input",
            "input file to read instead of stdin",
            "FILE",
        );
        opts.optopt(SHORT_OUTDIR, "outdir", "set output directory", "DIR");
        opts.optflag(SHORT_HELP, "help", "print this help menu");
        opts.optflag(SHORT_VERSION, "version", "print the version");

        let matches = opts.parse(&args[1..]).unwrap();
        if matches.opt_present(SHORT_HELP) {
            print_usage(&program, opts);
            return Ok(None);
        }
        if matches.opt_present(SHORT_VERSION) {
            print_version();
            return Ok(None);
        }
        let output_dir = matches
            .opt_str(SHORT_OUTDIR)
            .ok_or_else(|| err_msg("Output directory missing"))?;
        let input = read_input(matches.opt_str(SHORT_INPUT))?;

        Ok(Some(FormOpts { output_dir, input }))
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    println!("{}", opts.usage(&brief));
}

fn print_version() {
    println!("{} {}", crate::NAME, crate::VERSION);
}

fn read_input<P: AsRef<Path>>(input_file: Option<P>) -> Result<String, Error> {
    let mut input = String::new();
    match input_file {
        Some(file_name) => {
            let mut file = File::open(file_name)?;
            file.read_to_string(&mut input)?;
        }
        None => {
            io::stdin().read_to_string(&mut input)?;
        }
    }
    Ok(input)
}

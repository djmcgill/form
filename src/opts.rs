use getopts::Options;
use std::env;
use std::fs::File;
use std::io::{self, Result, Read};
use std::path::Path;

pub struct FormOpts {
    pub input: String,
    pub output_dir: String,
}

impl FormOpts {
    pub fn from_args() -> Result<Option<Self>> {
        const SHORT_INPUT: &'static str = "i";
        const SHORT_OUTDIR: &'static str = "o";
        const SHORT_HELP: &'static str = "h";

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

        let matches = opts.parse(&args[1..]).unwrap();
        if matches.opt_present("h") {
            print_usage(&program, opts);
            return Ok(None);
        }
        let output_dir = matches.opt_str(SHORT_OUTDIR).unwrap_or(
            "/Users/davidmcgillicuddy/private/code/form/src/test"
                .to_string(),
        );
        let input = read_input(matches.opt_str(SHORT_INPUT))?;

        Ok(Some(FormOpts {
            output_dir: output_dir,
            input: input,
        }))
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn read_input<P: AsRef<Path>>(input_file: Option<P>) -> Result<String> {
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

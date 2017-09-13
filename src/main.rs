extern crate getopts;
extern crate syn;
extern crate quote;

use getopts::Options;
use std::env;
use syn::{Crate, Item, ItemKind};
use syn::fold::*;
use quote::ToTokens;

use std::fs::{self, create_dir, File};
use std::io::{self, Write, Result, Read};
use std::path::{Path, PathBuf};
use std::str;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

struct FormOpts {
    force: bool,
    input: String,
    output_dir: String,
}

impl FormOpts {
    pub fn from_args() -> Result<Option<Self>> {
        const short_force: &'static str = "f";
        const short_input: &'static str = "i";
        const short_outdir: &'static str = "o";
        const short_help: &'static str = "h";

        let args: Vec<String> = env::args().collect();
        let program = args[0].clone();
        let mut opts = Options::new();
        opts.optflag(short_force, "force", "force the overwriting of the old lib.rs file");
        opts.optopt(short_input, "input", "input file to read instead of stdin", "FILE");
        opts.optopt(short_outdir, "outdir", "set output directory", "DIR");
        opts.optflag(short_help, "help", "print this help menu");

        let matches = opts.parse(&args[1..]).unwrap();
        if matches.opt_present("h") {
            print_usage(&program, opts);
            return Ok(None);
        }
        let output_dir = matches.opt_str(short_outdir).unwrap_or(
            "/Users/davidmcgillicuddy/private/code/form/src/test"
                .to_string(),
        );
        let force = matches.opt_present(short_force);
        let input = read_input(matches.opt_str(short_input))?;

        Ok(Some(FormOpts {
            output_dir: output_dir,
            input: input,
            force: force,
        }))
    }
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

pub fn main() {
    if let Ok(Some(opts)) = FormOpts::from_args() {
        create_directory_structure(opts.output_dir, opts.input, opts.force).unwrap();
    }
}

fn create_directory_structure<P: AsRef<Path>>(
    base_dir: P,
    string_contents: String,
    force: bool,
) -> Result<()> {
    let parsed_crate = syn::parse_crate(&string_contents).unwrap();

    let base_dir = base_dir.as_ref();
    if !base_dir.exists() {
        create_dir(base_dir)?;
    }
    let lib_file = base_dir.join("lib.rs");
    if lib_file.exists() {
        if force {
            fs::remove_file(&lib_file)?; // TODO: just overwrite instead
        } else {
            return Err(format!(
                "The lib.rs file {} already exists. Please delete it and try again.",
                &lib_file.display()
            ));
        }
    }

    let mut folder = FoldMods { current_dir: &base_dir };
    let new_contents = folder.fold_crate(parsed_crate);

    let mut file = File::create(lib_file)?;
    write_all_tokens(&new_contents, &mut file)
}

struct FoldMods<P: AsRef<Path>> {
    current_dir: P,
}
impl<P: AsRef<Path>> FoldMods<P> {
    fn sub_mod<Q: AsRef<Path>>(&self, path: Q) -> FoldMods<PathBuf> {
        FoldMods { current_dir: self.current_dir.as_ref().join(path) }
    }
}

impl<P: AsRef<Path>> Folder for FoldMods<P> {
    fn fold_item(&mut self, mut item: Item) -> Item {
        for rust_crate in extract_crate_from_mod(&mut item.node) {
            let crate_name = &item.ident;

            let dir_name = &self.current_dir.as_ref().join(crate_name.as_ref());
            create_dir(dir_name).unwrap();

            let mut sub_self = self.sub_mod(crate_name.as_ref());
            let folded_crate = noop_fold_crate(&mut sub_self, rust_crate);
            write_crate(folded_crate, &dir_name).unwrap();
        }
        noop_fold_item(self, item)
    }
}

fn write_crate<P: AsRef<Path>>(rust_crate: Crate, dir_name: &P) -> Result<()> {
    let file_name = dir_name.as_ref().join("mod.rs");
    let mut file = File::create(file_name)?;
    write_all_tokens(&rust_crate, &mut file)
}

fn extract_crate_from_mod<'a>(node: &'a mut ItemKind) -> Option<Crate> {
    if let ItemKind::Mod(ref mut maybe_items) = *node {
        maybe_items.take().map(make_crate)
    } else {
        None
    }
}

fn make_crate(items: Vec<Item>) -> Crate {
    Crate {
        shebang: None,
        attrs: vec![],
        items: items,
    }
}

fn write_all_tokens<T: ToTokens, W: Write>(piece: &T, writer: &mut W) -> Result<()> {
    let mut new_tokens = quote::Tokens::new();
    piece.to_tokens(&mut new_tokens);
    let string = new_tokens.into_string();
    writer.write_all(string.as_bytes())
}

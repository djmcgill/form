#![recursion_limit = "1024"]
extern crate getopts;
extern crate quote;
extern crate syn;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate failure;

use quote::ToTokens;
use syn::{Crate, Item, ItemKind, Ident};
use syn::fold::*;
use log::LevelFilter;

use std::fs::{DirBuilder, File};
use std::io::Write;
use std::path::{Path, PathBuf};

mod opts;
use opts::FormOpts;
use failure::*;

fn main() {
    match run() {
        Ok(()) => println!("Completed successfully"),
        Err(error) => println!("Failed with:\n {}", error),
    }
}

fn run() -> Result<(), Error> {
    env_logger::Builder::new().filter_level(LevelFilter::Info).try_init().context("could not initialise env_logger")?;

    trace!("logging initialised");
    let try_parsed_args = FormOpts::from_args().context(
        "could not parse the command line arguments"
    )?;
    // if None, we've already printed a help text and have nothing more to do
    if let Some(opts) = try_parsed_args {
        create_directory_structure(opts.output_dir, opts.input)?;
    }
    return Ok(())
}

fn create_directory_structure<P: AsRef<Path>>(
    base_dir: P,
    string_contents: String,
) -> Result<(), Error> {
    info!("Started parsing the input as Rust. This can take a minute or two.");
    let parsed_crate = syn::parse_crate(&string_contents).map_err(err_msg).context("failed to parse crate")?;
    info!("Finished parsing");

    let base_dir = base_dir.as_ref();
    let mut dir_builder = DirBuilder::new();
    dir_builder.recursive(true).create(&base_dir).context(
        format_err!("unable to create the directory {}", base_dir.display())
    )?;
    info!("Prepared target directory {}", base_dir.display());

    let mut folder = FileIntoMods { current_dir: &base_dir };

    // Why doesn't syn::Fold::fold handle errors again?
    // TODO: catch panics?
    let new_contents = folder.fold_crate(parsed_crate);
    trace!("transformed module contents");

    let lib_file_path = base_dir.join("lib.rs");

    let mut file = File::create(&lib_file_path).context(
        format_err!("Unable to create the file {}", lib_file_path.display())
    )?;
    debug!("Writing to file {}", lib_file_path.display());
    write_all_tokens(&new_contents, &mut file).context("unable to write to lib.rs")?;
    Ok(())
}

#[derive(Debug)]
struct FileIntoMods<P: AsRef<Path> + Send + Sync> {
    current_dir: P,
}
impl<P: AsRef<Path> + Send + Sync> FileIntoMods<P> {
    fn sub_mod<Q: AsRef<Path>>(&self, path: Q) -> FileIntoMods<PathBuf> {
        FileIntoMods {
            current_dir: self.current_dir.as_ref().join(path),
        }
    }
}

impl<P: AsRef<Path> + Send + Sync> FileIntoMods<P> {
    fn fold_sub_crate(&mut self, crate_name: &Ident, rust_crate: Crate) -> Result<(), Error> {
        trace!("Folding over module {}", crate_name);

        let dir_name = &self.current_dir.as_ref().join(crate_name.as_ref());

        let mut dir_builder = DirBuilder::new();
        info!("Creating directory {}", dir_name.display());
        dir_builder
            .recursive(true)
            .create(dir_name)
            .unwrap_or_else(|err| {
                panic!("building {} failed with {}", dir_name.display(), err)
            });

        let mut sub_self = self.sub_mod(crate_name.as_ref());
        let folded_crate = noop_fold_crate(&mut sub_self, rust_crate);
        trace!(
            "Writing contents of module {} to file {}",
            crate_name,
            dir_name.display()
        );
        write_crate(folded_crate, &dir_name).unwrap_or_else(|err| {
            panic!(
                "writing to {}/mod.rs failed with {}",
                dir_name.display(),
                err
            )
        });
        Ok(())
    }
}

impl<P: AsRef<Path> + Send + Sync> Folder for FileIntoMods<P> {
    fn fold_item(&mut self, mut item: Item) -> Item {
        for rust_crate in extract_crate_from_mod(&mut item.node) {
            self.fold_sub_crate(&item.ident, rust_crate).unwrap();
        }
        noop_fold_item(self, item)
    }
}

fn write_crate<P: AsRef<Path>>(rust_crate: Crate, dir_name: &P) -> Result<(), Error> {
    let file_name = dir_name.as_ref().join("mod.rs");
    trace!("Opening file {}", file_name.display());
    let mut file = File::create(&file_name).context(
        format_err!("unable to create file {}", file_name.display())
    )?;
    trace!("Successfully opened file {}", file_name.display());
    debug!("Writing to file {}", file_name.display());
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
        items,
    }
}

fn write_all_tokens<T: ToTokens, W: Write>(piece: &T, writer: &mut W) -> Result<(), Error> {
    let mut new_tokens = quote::Tokens::new();
    piece.to_tokens(&mut new_tokens);
    let string = new_tokens.into_string();
    trace!("Written string for tokens, now writing");
    writer.write_all(string.as_bytes()).context(
        "unable to write the tokens to the file",
    )?;
    trace!("Successfully wrote token string");
    Ok(())
}

#![recursion_limit = "1024"]
extern crate getopts;
extern crate quote;
extern crate syn;
#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate error_chain;

use quote::ToTokens;
use syn::{Crate, Item, ItemKind, Ident};
use syn::fold::*;

use std::fs::{DirBuilder, File};
use std::io::Write;
use std::path::{Path, PathBuf};

mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain!{}
}

use errors::*;

mod opts;
use opts::FormOpts;


quick_main!(run);

fn run() -> Result<()> {
    env_logger::init().chain_err(
        || "could not initialise env_logger",
    )?;
    debug!("logging initialised");
    if let Some(opts) = FormOpts::from_args().chain_err(
        || "could not parse the command line arguments",
    )?
    {
        create_directory_structure(opts.output_dir, opts.input)?;
    }
    Ok(())
}

fn create_directory_structure<P: AsRef<Path>>(
    base_dir: P,
    string_contents: String,
) -> Result<()> {
    let parsed_crate = syn::parse_crate(&string_contents).map_err(Error::from)?;

    let base_dir = base_dir.as_ref();
    let mut dir_builder = DirBuilder::new();
    dir_builder.recursive(true).create(&base_dir).chain_err(
        || {
            format!("unable to create the directory {}", base_dir.display())
        },
    )?;

    let mut folder = FoldMods {
        current_dir: &base_dir,
    };
    let new_contents = folder.fold_crate(parsed_crate);
    let lib_file_path = base_dir.join("lib.rs");

    let mut file = File::create(&lib_file_path).chain_err(|| {
        format!("Unable to create the file {}", lib_file_path.display())
    })?;
    write_all_tokens(&new_contents, &mut file).chain_err(|| "unable to write to lib.rs")
}

struct FoldMods<P: AsRef<Path>> {
    current_dir: P,
}
impl<P: AsRef<Path>> FoldMods<P> {
    fn sub_mod<Q: AsRef<Path>>(&self, path: Q) -> FoldMods<PathBuf> {
        FoldMods {
            current_dir: self.current_dir.as_ref().join(path),
        }
    }
}

impl<P: AsRef<Path>> FoldMods<P> {
    fn fold_sub_crate(&mut self, crate_name: &Ident, rust_crate: Crate) -> Result<()> {
        debug!("Folding over module {}", crate_name);

        let dir_name = &self.current_dir.as_ref().join(crate_name.as_ref());

        let mut dir_builder = DirBuilder::new();
        debug!("Creating directory {}", dir_name.display());
        dir_builder
            .recursive(true)
            .create(dir_name)
            .unwrap_or_else(|err| {
                panic!("building {} failed with {}", dir_name.display(), err)
            });

        let mut sub_self = self.sub_mod(crate_name.as_ref());
        let folded_crate = noop_fold_crate(&mut sub_self, rust_crate);
        debug!(
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

impl<P: AsRef<Path>> Folder for FoldMods<P> {
    fn fold_item(&mut self, mut item: Item) -> Item {
        for rust_crate in extract_crate_from_mod(&mut item.node) {
            self.fold_sub_crate(&item.ident, rust_crate).unwrap();
        }
        noop_fold_item(self, item)
    }
}

fn write_crate<P: AsRef<Path>>(rust_crate: Crate, dir_name: &P) -> Result<()> {
    let file_name = dir_name.as_ref().join("mod.rs");
    debug!("Opening file {}", file_name.display());
    let mut file = File::create(&file_name).chain_err(|| {
        format!("unable to create file {}", file_name.display())
    })?;
    debug!("Successfully opened file {}", file_name.display());
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
    debug!("Written string for tokens, now writing");
    writer.write_all(string.as_bytes()).chain_err(
        || "unable to write the tokens to the file",
    )?;
    debug!("Successfully wrote token string");
    Ok(())
}

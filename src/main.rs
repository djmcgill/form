extern crate getopts;
extern crate quote;
extern crate syn;
#[macro_use]
extern crate log;
extern crate env_logger;

use quote::ToTokens;
use syn::{Crate, Item, ItemKind};
use syn::fold::*;

use std::fs::{DirBuilder, File, OpenOptions};
use std::io::{Result, Write};
use std::path::{Path, PathBuf};

mod opts;
use opts::FormOpts;
pub fn main() {
    env_logger::init().unwrap_or_else(|err| panic!("env_logger initialisation failed with {}", err));
    debug!("logging initialised");
    if let Ok(Some(opts)) = FormOpts::from_args() {
        create_directory_structure(opts.output_dir, opts.input, opts.force).unwrap_or_else(|err|
            panic!("create_directory_structure failed with {}", err)
        );
    }
}

fn file_open_options(force: bool) -> OpenOptions {
    let mut write_open_opts = OpenOptions::new();
    write_open_opts.write(true);
    if !force {
        write_open_opts.create(true);
    }
    write_open_opts
}

fn create_directory_structure<P: AsRef<Path>>(
    base_dir: P,
    string_contents: String,
    force: bool,
) -> Result<()> {
    let parsed_crate = syn::parse_crate(&string_contents).unwrap_or_else(|err| panic!("crate parsing failed with {}", err));

    let base_dir = base_dir.as_ref();
    let mut dir_builder = DirBuilder::new();
    dir_builder.recursive(true).create(base_dir)?;

    let mut folder = FoldMods {
        current_dir: &base_dir,
        force: force,
    };
    let new_contents = folder.fold_crate(parsed_crate);
    let lib_file_path = base_dir.join("lib.rs");

    // let write_open_opts = file_open_options(force);
    // let mut file = write_open_opts.open(lib_file_path)?;
    let mut file = File::create(lib_file_path)?;
    write_all_tokens(&new_contents, &mut file)
}

struct FoldMods<P: AsRef<Path>> {
    current_dir: P,
    force: bool,
}
impl<P: AsRef<Path>> FoldMods<P> {
    fn sub_mod<Q: AsRef<Path>>(&self, path: Q) -> FoldMods<PathBuf> {
        FoldMods {
            current_dir: self.current_dir.as_ref().join(path),
            force: self.force,
        }
    }
}

impl<P: AsRef<Path>> Folder for FoldMods<P> {
    fn fold_item(&mut self, mut item: Item) -> Item {
        for rust_crate in extract_crate_from_mod(&mut item.node) {
            let crate_name = &item.ident;
            debug!("Folding over module {}", crate_name);

            let dir_name = &self.current_dir.as_ref().join(crate_name.as_ref());

            let mut dir_builder = DirBuilder::new();
            debug!("Creating directory {}", dir_name.display());
            dir_builder.recursive(true).create(dir_name).unwrap_or_else(|err| panic!("building {} failed with {}", dir_name.display(), err));

            let mut sub_self = self.sub_mod(crate_name.as_ref());
            let folded_crate = noop_fold_crate(&mut sub_self, rust_crate);
            debug!("Writing contents of module {} to file {}", crate_name, dir_name.display());
            write_crate(folded_crate, &dir_name, self.force).unwrap_or_else(|err| panic!("writing to {}/mod.rs failed with {}", dir_name.display(), err));
        }
        noop_fold_item(self, item)
    }
}

fn write_crate<P: AsRef<Path>>(rust_crate: Crate, dir_name: &P, force: bool) -> Result<()> {
    let file_name = dir_name.as_ref().join("mod.rs");
    // let open_options = file_open_options(force);
    debug!("Opening file {} with force={}", file_name.display(), force);
    // let mut file = open_options.open(&file_name)?;
    let mut file = File::create(&file_name)?;
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
    writer.write_all(string.as_bytes())?;
    debug!("Successfully wrote token string");
    Ok(())
}

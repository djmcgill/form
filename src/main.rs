extern crate syn;
extern crate quote;

use syn::{Crate, Item, ItemKind};
use syn::fold::*;
use quote::ToTokens;

use std::fs::{create_dir, File};
use std::io::{Write, Result};
use std::path::{Path, PathBuf};
use std::str;

pub fn main() {
    use std::io::Read; 
    let mut file = File::open("/Users/davidmcgillicuddy/private/code/form/src/small-lib.rs").unwrap();
    let mut buffer = vec![];
    file.read_to_end(&mut buffer).unwrap();
    create_directory_structure(
        "/Users/davidmcgillicuddy/private/code/form/src/test",
        &buffer
    );
}


fn create_directory_structure<P: AsRef<Path>>(base_dir: P, contents: &Vec<u8>) {
    let string_contents = str::from_utf8(contents.as_slice()).unwrap();
    let parsed_crate = syn::parse_crate(&string_contents).unwrap();

    let base_dir = base_dir.as_ref();
    if !base_dir.exists() {
        create_dir(base_dir).unwrap();
    }
    let lib_file = base_dir.join("lib.rs");
    assert!(!lib_file.exists(),
        format!("The lib.rs file {} already exists. Please delete it and try again.", lib_file.display())
    );

    let mut folder = FoldMods {
        current_dir: &base_dir
    };
    let new_contents = folder.fold_crate(parsed_crate);

    let mut file = File::create(lib_file).unwrap();
    write_all_tokens(&new_contents, &mut file).unwrap();
}

struct FoldMods<P> {
    current_dir: P
}
impl<P: AsRef<Path>> FoldMods<P> {
    fn sub_mod<Q: AsRef<Path>>(&self, path: Q) -> FoldMods<PathBuf> {
        FoldMods {
            current_dir: self.current_dir.as_ref().join(path)
        }
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
        };
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

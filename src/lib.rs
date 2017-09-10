extern crate syn;
extern crate quote;

use syn::{Crate, Item};
use syn::fold::Folder;
use quote::ToTokens;

use std::fs::{create_dir, File};
use std::io::{Write, Result};
use std::path::Path;
use std::str;

pub fn create_directory_structure<P: AsRef<Path>>(base_dir: P, contents: &Vec<u8>) {
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

struct FoldMods<'a, P: 'a> {
    current_dir: &'a P
}

impl<'a, P: AsRef<Path>> Folder for FoldMods<'a, P> {
    fn fold_item(&mut self, mut item: Item) -> Item {
        if let syn::ItemKind::Mod(ref mut maybe_items) = item.node {
            for items in maybe_items.take() {
                let crate_ = Crate {
                    shebang: None,
                    attrs: vec![],
                    items: items,
                };

                let crate_name = item.ident.as_ref();
                let dir_name = self.current_dir.as_ref().join(crate_name);
                let file_name = dir_name.join("mod.rs");
                create_dir(dir_name).unwrap();
                let mut file = File::create(file_name).unwrap();
                write_all_tokens(&crate_, &mut file).unwrap();
            }
        };
        item
    }
}

fn write_all_tokens<T: ToTokens, W: Write>(piece: &T, writer: &mut W) -> Result<()> {
    let mut new_tokens = quote::Tokens::new();
    piece.to_tokens(&mut new_tokens);
    let string = new_tokens.into_string();
    writer.write_all(string.as_bytes())
}

#[test]
fn name() {
    use std::io::Read; 
    let mut file = File::open("/Users/davidmcgillicuddy/private/code/form/src/small-lib.rs").unwrap();
    let mut buffer = vec![];
    file.read_to_end(&mut buffer).unwrap();
    create_directory_structure(
        "/Users/davidmcgillicuddy/private/code/form/src/test",
        &buffer
    );
}

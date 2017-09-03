use std::fs::File;
use std::io::Read;

extern crate proc_macro;
use proc_macro::TokenStream;

extern crate syn;

#[macro_use]
extern crate quote;

use quote::ToTokens;

pub fn my_macro(file_name: &str) {
    let mut file = File::open(file_name).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let file_contents = syn::parse_file(&contents).unwrap();
    
    println!("total items: {:?}", file_contents.items.len());

    let mut folder = FoldMods;
    let new_contents = folder.fold_file(file_contents);
    
    // use syn::printing::*;
    let mut new_tokens = quote::Tokens::new();
    new_contents.to_tokens(&mut new_tokens);
    /*
        error[E0599]: no method named `to_tokens` found for type `syn::File` in the current scope
        --> src/lib.rs:27:18
        |
        27 |     new_contents.to_tokens(&mut new_tokens);
        |                  ^^^^^^^^^
        |
        = help: items from traits can only be used if the trait is in scope
        = note: the following trait is implemented but not in scope, perhaps add a `use` for it:
                candidate #1: `use quote::to_tokens::ToTokens;`

    */
    println!("{}", new_tokens);
}

struct FoldMods;
use syn::Item;
use syn::fold::Folder;
impl syn::fold::Folder for FoldMods {
    fn fold_item(&mut self, item: Item) -> Item {
        match item.node {
            syn::ItemKind::Mod(item_mod) =>
                syn::Item {
                    node: syn::ItemKind::Mod(on_mod(&item_mod)),
                    .. item
                },
            _ => item,
        }
    }
}

fn on_mod(item_mod: &syn::ItemMod) -> syn::ItemMod {
    syn::ItemMod {
        content: None,
        .. item_mod.clone()
    }
}

#[test]
fn name() {
    my_macro("/Users/davidmcgillicuddy/private/code/form/src/small-lib.rs");
    panic!();
}

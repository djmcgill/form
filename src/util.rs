use quote::ToTokens;
use syn::fold::*;
use syn::{Ident, Item};

use std::fs::{DirBuilder, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use failure::*;

pub fn create_directory_structure<P: AsRef<Path>>(
    base_dir: P,
    string_contents: &str,
) -> Result<(), Error> {
    info!("Started parsing the input as Rust. This can take a minute or two.");
    let parsed_crate = syn::parse_file(&string_contents)
        .map_err(err_msg)
        .context("failed to parse crate")?;
    info!("Finished parsing");

    let base_dir = base_dir.as_ref();
    let mut dir_builder = DirBuilder::new();
    dir_builder
        .recursive(true)
        .create(&base_dir)
        .context(format_err!(
            "unable to create the directory {}",
            base_dir.display()
        ))?;
    info!("Prepared target directory {}", base_dir.display());

    let mut folder = FileIntoMods {
        current_dir: &base_dir,
        top_level: true,
    };

    // Why doesn't syn::Fold::fold handle errors again?
    // TODO: catch panics?
    let new_contents = folder.fold_file(parsed_crate);
    trace!("transformed module contents");

    let lib_file_path = base_dir.join("lib.rs");

    let mut file = File::create(&lib_file_path).context(format_err!(
        "Unable to create the file {}",
        lib_file_path.display()
    ))?;
    debug!("Writing to file {}", lib_file_path.display());
    write_all_tokens(&new_contents, &mut file).context("unable to write to lib.rs")?;
    Ok(())
}

#[derive(Debug)]
struct FileIntoMods<P: AsRef<Path> + Send + Sync> {
    current_dir: P,
    top_level: bool,
}
impl<P: AsRef<Path> + Send + Sync> FileIntoMods<P> {
    fn sub_mod<Q: AsRef<Path>>(&self, path: Q) -> FileIntoMods<PathBuf> {
        FileIntoMods {
            current_dir: self.current_dir.as_ref().join(path),
            top_level: false,
        }
    }
}

impl<P: AsRef<Path> + Send + Sync> FileIntoMods<P> {
    fn fold_sub_mod(&mut self, mod_name: Ident, mod_file: syn::File) -> Result<(), Error> {
        let mod_name = mod_name.to_string();
        trace!("Folding over module {}", mod_name);

        if !self.current_dir.as_ref().exists() {
            let mut dir_builder = DirBuilder::new();
            info!("Creating directory {}", self.current_dir.as_ref().display());
            dir_builder
                .recursive(true)
                .create(self.current_dir.as_ref())
                .unwrap_or_else(|err| {
                    panic!(
                        "building {} failed with {}",
                        self.current_dir.as_ref().display(),
                        err
                    )
                });
        }

        let mut sub_self = self.sub_mod(&mod_name);
        let folded_mod = fold_file(&mut sub_self, mod_file);
        let file_name = self
            .current_dir
            .as_ref()
            .join(mod_name.to_owned() + ".rs");
        trace!(
            "Writing contents of module {} to file {}",
            mod_name,
            file_name.display()
        );
        write_mod_file(folded_mod, &file_name)
            .unwrap_or_else(|err| panic!("writing to {} failed with {}", file_name.display(), err));
        Ok(())
    }
}

impl<P: AsRef<Path> + Send + Sync> Fold for FileIntoMods<P> {
    fn fold_item(&mut self, mut item: Item) -> Item {
        if let Some((mod_name, mod_file)) = extract_mod(&mut item, self.top_level) {
            self.fold_sub_mod(mod_name, mod_file).unwrap();
        }
        fold_item(self, item)
    }
}

fn write_mod_file(item_mod: syn::File, file_name: &Path) -> Result<(), Error> {
    trace!("Opening file {}", file_name.display());
    let mut file = File::create(&file_name)
        .context(format_err!("unable to create file {}", file_name.display()))?;
    trace!("Successfully opened file {}", file_name.display());
    debug!("Writing to file {}", file_name.display());
    write_all_tokens(&item_mod, &mut file)
}

fn extract_mod(node: &mut Item, top_level: bool) -> Option<(Ident, syn::File)> {
    if let Item::Mod(mod_item) = &mut *node {
        if let Some(item_content) = mod_item.content.take() {
            let items = item_content.1;
            Some((mod_item.ident.clone(), make_file(items)))
        } else if top_level {
            None
        } else {
            panic!("Moving nested non-inline `mod` declarations not currently supported.")
        }
    } else {
        None
    }
}

fn make_file(items: Vec<Item>) -> syn::File {
    syn::File {
        shebang: None,
        attrs: vec![],
        items,
    }
}

fn write_all_tokens<T: ToTokens, W: Write>(piece: &T, writer: &mut W) -> Result<(), Error> {
    let mut new_tokens = proc_macro2::TokenStream::new();
    piece.to_tokens(&mut new_tokens);
    let string = new_tokens.to_string();
    trace!("Written string for tokens, now writing");
    writer
        .write_all(string.as_bytes())
        .context("unable to write the tokens to the file")?;
    trace!("Successfully wrote token string");
    Ok(())
}

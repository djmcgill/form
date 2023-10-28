use anyhow::{Context, Result};
use log::{debug, info, trace};
use quote::ToTokens;
use std::{
    fs::{DirBuilder, File},
    io::Write,
    path::{Path, PathBuf},
};
use syn::{fold::*, parse_quote, Ident, Item};

pub fn create_directory_structure<P: AsRef<Path>>(
    base_dir: P,
    string_contents: &str,
) -> Result<()> {
    info!("Started parsing the input as Rust. This can take a minute or two.");
    let parsed_crate = syn::parse_file(string_contents).context("failed to parse crate")?;
    info!("Finished parsing");

    let base_dir = base_dir.as_ref();
    let mut dir_builder = DirBuilder::new();
    dir_builder
        .recursive(true)
        .create(&base_dir)
        .with_context(|| format!("unable to create the directory {}", base_dir.display()))?;
    info!("Prepared target directory {}", base_dir.display());

    let mut folder = FileIntoMods {
        current_dir: &base_dir,
        current_mod_fs_name: None,
        has_path_attr: false,
    };

    // Why doesn't syn::Fold::fold handle errors again?
    // TODO: catch panics?
    let new_contents = folder.fold_file(parsed_crate);
    trace!("transformed module contents");

    let lib_file_path = base_dir.join("lib.rs");

    let mut file = File::create(&lib_file_path)
        .with_context(|| format!("Unable to create the file {}", lib_file_path.display()))?;
    debug!("Writing to file {}", lib_file_path.display());
    write_all_tokens(&new_contents, &mut file).context("unable to write to lib.rs")?;
    Ok(())
}

#[derive(Debug)]
struct FileIntoMods<P: AsRef<Path> + Send + Sync> {
    current_dir: P,
    current_mod_fs_name: Option<String>,
    has_path_attr: bool,
}
impl<P: AsRef<Path> + Send + Sync> FileIntoMods<P> {
    fn sub_mod(&self, path: String, has_path_attr: bool) -> FileIntoMods<PathBuf> {
        FileIntoMods {
            current_dir: self.current_dir.as_ref().join(&path),
            current_mod_fs_name: Some(path),
            has_path_attr,
        }
    }
}

impl<P: AsRef<Path> + Send + Sync> FileIntoMods<P> {
    fn fold_sub_mod(
        &mut self,
        mod_name: Ident,
        mod_fs_name: String,
        mod_has_path_attr: bool,
        mod_file: syn::File,
    ) -> Result<()> {
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

        let mut sub_self = self.sub_mod(mod_fs_name.clone(), mod_has_path_attr);
        let folded_mod = fold_file(&mut sub_self, mod_file);
        let file_name = self.current_dir.as_ref().join(mod_fs_name + ".rs");
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
        if let Some((mod_name, mod_fs_name, mod_has_path_attr, mod_file)) =
            extract_mod(&mut item, &self.current_mod_fs_name, self.has_path_attr)
        {
            self.fold_sub_mod(mod_name, mod_fs_name, mod_has_path_attr, mod_file)
                .unwrap();
        }
        fold_item(self, item)
    }
}

fn write_mod_file(item_mod: syn::File, file_name: &Path) -> Result<()> {
    trace!("Opening file {}", file_name.display());
    let mut file = File::create(&file_name)
        .with_context(|| format!("unable to create file {}", file_name.display()))?;
    trace!("Successfully opened file {}", file_name.display());
    debug!("Writing to file {}", file_name.display());
    write_all_tokens(&item_mod, &mut file)
}

fn extract_mod(
    node: &mut Item,
    parent_fs_name: &Option<String>,
    parent_has_path_attr: bool,
) -> Option<(Ident, String, bool, syn::File)> {
    let top_level = parent_fs_name.is_none();
    if let Item::Mod(mod_item) = &mut *node {
        if let Some(item_content) = mod_item.content.take() {
            let items = item_content.1;

            let mod_name = mod_item.ident.clone();
            let mod_name_str = mod_name.to_string();

            let mod_name_is_reserved = is_name_reserved(&mod_name_str);
            let mod_has_path_attr = parent_has_path_attr || mod_name_is_reserved;
            let mod_fs_name = if mod_name_is_reserved {
                xform_reserved_name(&mod_name_str)
            } else {
                mod_name_str
            };

            if mod_has_path_attr {
                let path_attr_val = match parent_fs_name {
                    Some(parent_fs_name) => format!("{parent_fs_name}/{mod_fs_name}.rs"),
                    None => format!("{mod_fs_name}.rs"),
                };
                mod_item
                    .attrs
                    .push(parse_quote! { #[path = #path_attr_val] });
            }

            Some((mod_name, mod_fs_name, mod_has_path_attr, make_file(items)))
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

fn write_all_tokens<T: ToTokens, W: Write>(piece: &T, writer: &mut W) -> Result<()> {
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

const RESERVED_NAMES: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM0", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
    "COM8", "COM9", "LPT0", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

fn is_name_reserved(name: &str) -> bool {
    RESERVED_NAMES
        .iter()
        .any(|&reserved_name| name.eq_ignore_ascii_case(reserved_name))
}

fn xform_reserved_name(reserved_name: &str) -> String {
    format!("{reserved_name}_")
}

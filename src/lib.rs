//! Librarian - a Rust crate for downloading and linking to non-rust libraries from app build scripts

#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]
#![warn(clippy::all)]

use std::{
    env,
    fs::{copy, read_dir},
    path::{Path, PathBuf},
};

#[cfg(feature = "download")]
mod download;
#[cfg(feature = "download")]
pub use download::*;

/// Get assumed path to the target executable directory (only works from a build script)
fn get_target_dir() -> PathBuf {
    // Please tell me there's a better way... please...
    let cur_exe = env::current_exe().unwrap();
    cur_exe.parent().unwrap().parent().unwrap().parent().unwrap().to_path_buf()
}

/// Get dynamic lib extension (.dll on windows, .so otherwise)
fn get_dylib_extension() -> &'static str {
    if cfg!(windows) {
        "dll"
    } else {
        "so"
    }
}

/// Install a single dynamic library with the given name and extension from the given path to the provided directory
pub fn install_dylib_with_extension_to(from: &Path, dylib_name: &str, extension: &str, target_dir: &Path) {
    let filename = dylib_name.to_string() + "." + extension;
    let path = from.join(filename.clone());
    if !path.exists() {
        panic!("Provided dylib doesn't exist!");
    }
    copy(path, target_dir.join(filename)).expect("Failed to copy dylib");
}

/// Install a single dynamic library with the given name from the given path to the provided directory
pub fn install_dylib_to(from: &Path, dylib_name: &str, target_dir: &Path) {
    install_dylib_with_extension_to(from, dylib_name, get_dylib_extension(), target_dir)
}

/// Install a single dynamic library with the given name from the given path to the target executable directory
pub fn install_dylib(from: &Path, dylib_name: &str) {
    install_dylib_to(from, dylib_name, get_target_dir().as_path())
}

/// Install all dynamic libs with given extension from the given path to the provided directory
pub fn install_dylibs_with_extension_to(from: &Path, extension: &str, target_dir: &Path) {
    let extension = ".".to_string() + extension;
    for entry in read_dir(from).expect("Can't read dylib source dir")  {
        let entry_path = entry.expect("Invalid fs entry").path();
        if let Some(file_name) = entry_path.file_name() {
            let file_name = file_name.to_str().unwrap();
            if file_name.ends_with(&extension) {
                copy(&entry_path, target_dir.join(file_name)).expect("Failed to copy dylib");
            }
        }
    }
}

/// Install all dynamic libs from the given path to the provided directory
pub fn install_dylibs_to(from: &Path, target_dir: &Path) {
    install_dylibs_with_extension_to(from, get_dylib_extension(), target_dir)
}  

/// Install all dynamic libs from the given path to the target executable directory
pub fn install_dylibs(from: &Path) {
    install_dylibs_to(from, get_target_dir().as_path())
}

/// Add a cargo link search path (only works strictly from a build script)
pub fn add_link_search_path(path: &Path) {
    println!("cargo:rustc-link-search=all={}", path.display());
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

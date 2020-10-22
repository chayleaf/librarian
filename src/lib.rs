//! Librarian - a Rust crate for downloading and linking to non-rust libraries from app build scripts

#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]
#![warn(clippy::all)]

use std::{
    env,
    error,
    fmt,
    fs,
    io,
    path::{Path, PathBuf},
};

#[cfg(feature = "web")]
mod web;
#[cfg(feature = "web")]
pub use web::*;

#[cfg(feature = "tgz")]
mod tgz;

#[cfg(feature = "zip")]
mod zip;

/// A type specifying an error that occured during an archive extraction
#[derive(Debug)]
pub enum ExtractError {
    /// Failed to read the zip file
    #[cfg(feature = "zip")]
    ZipError(rc_zip::Error),
    /// Failed to save files from the archive to the HDD
    WriteError(io::Error),
}

impl From<io::Error> for ExtractError {
    #[inline]
    fn from(err: io::Error) -> ExtractError {
        ExtractError::WriteError(err)
    }
}

impl fmt::Display for ExtractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ExtractError::*;
        match *self {
            #[cfg(feature = "zip")]
            ZipError(ref e) => e.fmt(f),
            WriteError(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for ExtractError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use ExtractError::*;
        match *self {
            #[cfg(feature = "zip")]
            ZipError(ref e) => Some(e),
            WriteError(ref e) => Some(e),
        }
    }
}

/// Extract the archive to a folder and return the path to the extracted files. Zip/tar/tar.gz archives are supported.
/// If extraction target path isn't provided and the function is executed from a build script, the build script output folder is used.
/// 
/// ```
/// # fn run() -> Result<(), Box<dyn std::error::Error>> {
/// # let path_to_lib_zip = std::path::Path::new("./whatever.zip");
/// let path_to_dylib_folder = librarian::extract_archive(path_to_lib_zip, None)?.join("bin");
/// librarian::install_dylibs(&path_to_dylib_folder, None, None)?;
/// # Ok(())
/// # }
/// ```
pub fn extract_archive<T: AsRef<Path> + ?Sized>(
    archive: &T,
    target: Option<&Path>
) -> Result<PathBuf, ExtractError> {
    let target = if let Some(target) = target {
        PathBuf::from(target)
    } else {
        PathBuf::from(env::var("OUT_DIR").expect("You must provide the output directory when not running from a build script."))
    };

    let fn_as_str = archive.as_ref().file_name().unwrap().to_string_lossy();
    #[cfg(feature = "zip")]
    {
        if fn_as_str.ends_with(".zip") {
            crate::zip::extract_zip(archive, target.as_path())?;
            return Ok(target);
        }
    }
    #[cfg(feature = "tgz")]
    {
        if fn_as_str.ends_with(".tar.gz") || fn_as_str.ends_with(".tgz") {
            crate::tgz::extract_tar_gz(archive, target.as_path())?;
            return Ok(target);
        }
    }
    #[cfg(feature = "tar")]
    {
        if fn_as_str.ends_with(".tar") {
            crate::tgz::extract_tar(archive, target.as_path())?;
            return Ok(target);
        }
    }
    
    panic!("archive format not supported");
}

/// Get assumed path to the target executable directory. Only works from build scripts.
fn get_target_dir() -> io::Result<PathBuf>  {
    // Please tell me there's a better way... please...
    let cur_exe = env::current_exe()?;
    Ok(cur_exe.parent().unwrap().parent().unwrap().parent().unwrap().to_path_buf())
}

/// Get dynamic lib extension (.dll for windows targets, .so otherwise). Only works from build scripts.
fn get_dylib_extension() -> Result<&'static str, env::VarError> {
    let target = env::var("TARGET")?;
    Ok(if target.contains("pc-windows") {
        "dll"
    } else {
        "so"
    })
}

/// Dynamic library filter used to specify which library files needs to be copied.
#[derive(Debug)]
pub enum DyLibNameFilter<'a> {
    /// Filename must match the string (Example: `"SDL2.dll"`)
    FileName(&'a str),
    /// Extension must match the string (Example: `"dll"`)
    Extension(&'a str),
    /// Library name must match the string (Example: `"SDL2"`).
    /// Extension will be inferred from the target platform.
    /// Files with an additional "lib" prefix will match as well.
    LibName(&'a str),
}

/// Install all dynamic libs from a directory to the target directory.
/// 
/// The `dylib` argument can be used to specify the criteria a file needs to match to be installed (See [DyLibNameFilter](DyLibNameFilter) docs).
/// Default behavior is to install every library with an extension that matches the target platform's dylib extension.
/// `target_dir` can be left empty to attempt to automatically find the target executable directory.
/// 
/// To just install all of the dynamic libraries from a folder, do:
/// 
/// ```
/// # fn run() -> std::io::Result<()> {
/// # let path_to_dylib_folder = std::path::Path::new(".");
/// librarian::install_dylibs(path_to_dylib_folder, None, None)?;
/// // The application should now have all the dynamic libraries in the same folder as the executable
/// # Ok(())
/// # }
/// ```
pub fn install_dylibs<T: AsRef<Path> + ?Sized>(
    from: &T,
    filter: Option<DyLibNameFilter<'_>>,
    target_dir: Option<&Path>,
) -> io::Result<()> {
    use DyLibNameFilter::*;

    let extension = if let Some(Extension(extension)) = filter {
        ".".to_string() + extension
    } else if let Some(FileName(_)) = filter {
        String::new()
    } else {
        get_dylib_extension().expect("Couldn't detect dylib extension").to_string()
    };

    let target_dir = if let Some(target_dir) = target_dir {
        PathBuf::from(target_dir)
    } else {
        get_target_dir()?
    };

    for entry in fs::read_dir(from)?  {
        let entry_path = entry?.path();
        if let Some(file_name) = entry_path.file_name() {
            if let Some(file_name) = file_name.to_str() {
                let matches = if let Some(FileName(target_fname)) = filter {
                    target_fname == file_name
                } else if let Some(Extension(_)) = filter {
                    file_name.ends_with(extension.as_str())
                } else if let Some(LibName(lib_name)) = filter {
                    file_name == lib_name.to_string() + &extension || file_name == "lib".to_string() + lib_name + &extension
                } else {
                    file_name.ends_with(extension.as_str())
                };

                if matches {
                    fs::copy(entry_path.as_path(), target_dir.join(file_name).as_path())?;
                }
            }
        }
    }
    Ok(())
}

/// Add a cargo link search path (only works strictly from a build script)
/// 
/// The function can be considered an analog of `install_dylibs` for static libs; it makes the static libs in a folder available to the linker.
/// 
/// Usage:
/// 
/// ```
/// # fn run() {
/// # let path_to_static_lib_folder = std::path::Path::new(".");
/// librarian::add_link_search_path(path_to_static_lib_folder);
/// # }
/// ```
pub fn add_link_search_path<T: AsRef<Path> + ?Sized>(path: &T) {
    println!("cargo:rustc-link-search=all={}", path.as_ref().display());
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        fs,
        path::Path,
        vec::Vec,
    };

    pub(crate) fn dir_list_equals(path: &Path, list: Vec<&'static str>) -> bool {
        let mut results = HashMap::new();
        for entry in fs::read_dir(path).unwrap() {
            let entry_path = entry.unwrap().path();
            let file_name = entry_path.file_name().unwrap().to_str().unwrap();
            *results.entry(file_name.to_string()).or_insert(0) += 1;
        }
        list.len() == results.len() && list.iter().all(|&x| *results.entry(x.to_string()).or_insert(0) == 1)
    }

    #[test]
    fn install_dylibs_test() {
        let cur_file = Path::new(file!());
        let root = cur_file.parent().unwrap().parent().unwrap();
        // For some reason, you can't just receive a temporary directory from cargo, you gotta manage it yourself
        let out = root.join("target").join("test").join("install_dylibs");
        let _ = fs::remove_dir_all(out.as_path());
        let data_dir = root.join("test_input");
        let dll_out = out.join("dll");
        let so_out = out.join("so");
        let fn_out = out.join("fn");
        fs::create_dir_all(dll_out.as_path()).unwrap();
        fs::create_dir_all(so_out.as_path()).unwrap();
        fs::create_dir_all(fn_out.as_path()).unwrap();

        use crate::*;
        use crate::DyLibNameFilter::*;
        install_dylibs(data_dir.as_path(), Some(Extension("dll")), Some(dll_out.as_path())).unwrap();
        install_dylibs(data_dir.as_path(), Some(Extension("so")), Some(so_out.as_path())).unwrap();
        install_dylibs(data_dir.as_path(), Some(FileName("dummy")), Some(fn_out.as_path())).unwrap();

        assert_eq!(true, dir_list_equals(dll_out.as_path(), vec![ "dummy0.dll", "dummy1.dll" ]));
        assert_eq!(true, dir_list_equals(so_out.as_path(), vec![ "dummy.so", "libdummy.so" ]));
        assert_eq!(true, dir_list_equals(fn_out.as_path(), vec![ "dummy" ]));
    }
}

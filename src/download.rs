#![cfg(feature = "download")]

use {
    bytes::Buf,
    positioned_io,
    rc_zip::{
        prelude::*,
        EntryContents,
    },
    std::{
        env,
        fs::File,
        io::copy,
        path::{
            Path,
            PathBuf,
        },
    },
    url::Url,
};

/// Get filename from URL
fn url_fname(url: &Url) -> Option<&str> {
    url
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
}

/// Download file to the provided directory and return the file's location
pub async fn download_or_find_file_to(url: &str, out_dir: &Path) -> PathBuf {
    let url_parsed = Url::parse(url).expect("Invalid URL");
    let fname = url_fname(&url_parsed).expect("Unknown filename");
    let response = reqwest::get(url).await.expect("Failed to download file");
        
    let path = PathBuf::from(out_dir).join(fname);
    if !path.exists() {
        let mut dest = File::create(path.clone()).expect("Failed to create file");
        let content = response.bytes().await.expect("Failed to get response bytes");
        let mut bytes = content.bytes();
        copy(&mut bytes, &mut dest).unwrap();
    }
    path
}

/// Download file to the build script output directory and return the file's location
pub async fn download_or_find_file(url: &str) -> PathBuf {
    let out_dir = env::var("OUT_DIR").expect("This function can only be used in a build script! Consider using download_or_find_file_with_path instead.");
    let path = Path::new(out_dir.as_str());
    download_or_find_file_to(url, path).await
}

/// Extract the archive to the provided directory
pub fn extract_archive_to(archive: &Path, dir: &Path) {
    if archive.file_name().unwrap().to_string_lossy().ends_with(".zip") {
        let zipfile = File::open(archive).unwrap();
        let reader = zipfile.read_zip().expect("Failed to read zip");
        for entry in reader.entries() {
            let dir = PathBuf::from(dir);
            match entry.contents() {
                EntryContents::Directory(c) => {
                    let path = dir.join(c.entry.name());
                    std::fs::create_dir_all(path.parent().unwrap()).expect("Failed to create dir");
                },
                EntryContents::File(c) => {
                    let path = dir.join(c.entry.name());
                    std::fs::create_dir_all(path.parent().unwrap()).expect("Failed to create dir");
                    let mut entry_writer = File::create(path).expect("Failed to create path");
                    let mut entry_reader = c
                        .entry
                        .reader(|offset| positioned_io::Cursor::new_pos(&zipfile, offset));

                    copy(&mut entry_reader, &mut entry_writer).expect("Copy from zip file failed");
                },
                _ => {}
            }
        }
    } else {
        panic!("archive format not supported yet");
    }
}

/// Extract the archive to the build script output directory and return the said directory location
pub fn extract_archive(archive: &Path) -> PathBuf {
    let path = PathBuf::from(env::var("OUT_DIR").expect("This function can only be used in a build script! Consider using extract_archive_to instead."));
    extract_archive_to(archive, &path);
    path
}
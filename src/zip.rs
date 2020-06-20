#![cfg(feature = "zip")]

use {
    crate::ExtractError,
    rc_zip::{
        prelude::*,
        EntryContents,
    },
    std::{
        fs,
        io,
        path::Path,
    },
};

impl From<rc_zip::Error> for ExtractError {
    #[inline]
    fn from(err: rc_zip::Error) -> ExtractError {
        ExtractError::ZipError(err)
    }
}

pub(crate) fn extract_zip<T: AsRef<Path> + ?Sized>(
    archive: &T,
    target: &Path
) -> Result<(), ExtractError> {
    let zipfile = fs::File::open(archive)?;
    let reader = zipfile.read_zip()?;
    for entry in reader.entries() {
        match entry.contents() {
            EntryContents::Directory(c) => {
                let path = target.join(c.entry.name());
                fs::create_dir_all(path.parent().unwrap())?;
            },
            EntryContents::File(c) => {
                let path = target.join(c.entry.name());
                fs::create_dir_all(path.parent().unwrap())?;
                let mut writer = fs::File::create(path)?;
                let mut reader = c
                    .entry
                    .reader(|offset| positioned_io::Cursor::new_pos(&zipfile, offset));

                io::copy(&mut reader, &mut writer)?;
            },
            // Symlinks aren't supported! Open an issue if you need them.
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod zip_tests {
    #[test]
    fn unzip() {
        use crate::tests::dir_list_equals;
        use crate::*;
        let cur_file = Path::new(file!());
        let root = cur_file.parent().unwrap().parent().unwrap();
        let out = root.join("target").join("test").join("unzip");
        let _ = fs::remove_dir_all(out.as_path());
        let data_dir = root.join("test_input");
        assert_eq!(extract_archive(data_dir.join("file.zip").as_path(), Some(out.as_path())).unwrap(), out);
        assert_eq!(true, dir_list_equals(out.join("zip").as_path(), vec![ "compressed.txt" ]));
        assert_eq!(
            fs::read_to_string(out.join("zip").join("compressed.txt")).unwrap(),
            "it works!".to_string()
        );
    }
}
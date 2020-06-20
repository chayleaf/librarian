#![cfg(feature = "tar")]

use std::{
    fs,
    path::Path,
};

#[cfg(feature = "tgz")]
pub(crate) fn extract_tar_gz<T: AsRef<Path> + ?Sized>(
    archive: &T,
    target: &Path
) -> Result<(), crate::ExtractError> {
    use flate2::read::GzDecoder;
    let tar_gz = fs::File::open(archive)?;
    let gz_decoder = GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(gz_decoder);
    archive.unpack(target)?;
    Ok(())
}

pub(crate) fn extract_tar<T: AsRef<Path> + ?Sized>(
    archive: &T,
    target: &Path
) -> Result<(), crate::ExtractError> {
    let file = fs::File::open(archive)?;
    let mut archive = tar::Archive::new(file);
    archive.unpack(target)?;
    Ok(())
}

#[cfg(feature = "tgz")]
#[cfg(test)]
mod tgz_tests {
    #[test]
    fn untgz() {
        use crate::tests::dir_list_equals;
        use crate::*;
        let cur_file = Path::new(file!());
        let root = cur_file.parent().unwrap().parent().unwrap();
        let out = root.join("target").join("test").join("untgz");
        let _ = fs::remove_dir_all(out.as_path());
        let data_dir = root.join("test_input");
        assert_eq!(extract_archive(data_dir.join("file.tgz").as_path(), Some(out.as_path())).unwrap(), out);
        assert_eq!(true, dir_list_equals(out.join("tgz").as_path(), vec![ "compressed.txt" ]));
        assert_eq!(
            fs::read_to_string(out.join("tgz").join("compressed.txt")).unwrap(),
            "this works as well".to_string()
        );
    }
}
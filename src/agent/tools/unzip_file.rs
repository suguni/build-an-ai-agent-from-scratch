use std::fs::File;
use std::io::Error;
use std::io::ErrorKind::NotFound;
use std::iter::Zip;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::{fs, io};
use zip::ZipArchive;
use zip::read::ZipFile;
use zip::result::{ZipError, ZipResult};

fn unzip_file(zip_path: &Path, extract_to: &Path) -> io::Result<String> {
    if !zip_path.exists() {
        return Err(Error::new(NotFound, "zip file not found"));
    }

    fs::create_dir(extract_to)?;

    let mut zip_archive = File::open(zip_path)
        .map_err(ZipError::from)
        .and_then(ZipArchive::new)
        .map_err(Error::from)?;

    let mut file_list: Vec<PathBuf> = vec![];

    for i in 0..zip_archive.len() {
        let mut file = zip_archive.by_index(i)?;
        let out_path = file
            .enclosed_name()
            .ok_or_else(|| Error::new(NotFound, "zip file not found"))?;

        let out_path = extract_to.join(out_path);

        if file.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(p) = out_path.parent()
                && !p.exists()
            {
                fs::create_dir_all(p)?;
            }
            File::create(&out_path).and_then(|mut out_file| io::copy(&mut file, &mut out_file))?;
        }

        file_list.push(out_path);
    }

    let mut result = format!(
        "Extracted {} files to {}\n\nContents:\n",
        file_list.len(),
        extract_to.to_str().unwrap()
    );

    result.push_str(
        &file_list
            .iter()
            .take(20)
            .map(|f| format!("  - {}\n", f.to_str().unwrap()))
            .collect::<String>(),
    );

    if file_list.len() > 20 {
        result.push_str(&format!("  ... and {} more files\n", file_list.len() - 20));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unzip_file() {
        let zip_path = Path::new("src.zip");
        let extract_to = Path::new("/tmp/src");

        let message = unzip_file(zip_path, extract_to).unwrap();

        println!("{message}");

        assert!(message.starts_with("Extracted"));
    }
}
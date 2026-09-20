use std::path::Path;
use std::{fs, io};

pub fn list_files(path: &Path) -> io::Result<String> {
    let mut dirs = vec![];
    let mut files = vec![];

    for entry in fs::read_dir(path)? {
        entry.map(|entry| {
            let path = entry.path();
            let file_name = entry.file_name();
            if path.is_dir() {
                dirs.push(file_name);
            } else if path.is_file() {
                files.push(file_name);
            }
        })?;
    }

    let mut result = format!("Directory: {}\n", path.to_str().unwrap());
    result.push_str(
        &[&dirs[..], &files[..]]
            .concat()
            .iter()
            .map(|s| format!("  {}", s.to_str().unwrap()))
            .collect::<Vec<_>>()
            .join("\n"),
    );

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_files() {
        let result = list_files(&Path::new("src")).unwrap();

        println!("{result}");

        assert!(result.starts_with("Directory: "));
    }
}
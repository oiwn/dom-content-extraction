use std::{fs, path};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomContentError {
    #[error("Error reading file")]
    UnableToReadFile(#[from] std::io::Error),
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub fn load_file(file_path: impl AsRef<path::Path>) -> Result<String, DomContentError> {
    let content: String =
        fs::read_to_string(file_path).map_err(DomContentError::UnableToReadFile)?;
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_load_file() {
        let content = load_file("html/sas-bankruptcy-protection.html");
        assert_eq!(content.is_ok(), true)
    }
}

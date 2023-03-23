use scraper::Html;
use std::{fs, path};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomContentError {
    #[error("Error reading file")]
    UnableToReadFile(#[from] std::io::Error),
}

#[allow(dead_code)]
pub fn read_file(
    file_path: impl AsRef<path::Path>,
) -> Result<String, DomContentError> {
    let content: String =
        fs::read_to_string(file_path).map_err(DomContentError::UnableToReadFile)?;
    Ok(content)
}

#[allow(dead_code)]
pub fn build_dom(html: &str) -> Html {
    let document: Html = Html::parse_document(html);
    document
}

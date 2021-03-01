use crate::markdown::Markdown;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::io::Read;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct IndexFile {
    path: String,
    content: String,
}

impl Markdown for IndexFile {
    fn content(&self) -> &str {
        &self.content
    }

    fn path(&self) -> &str {
        &self.path
    }
}

impl IndexFile {
    fn from_str(content: &str, path: &str) -> Self {
        let mut path = path.strip_suffix(".md").unwrap().to_string();
        path.push_str(".htmlpart");
        Self {
            path,
            content: content.to_string(),
        }
    }
    pub fn from_file(mut file: File, relative_path: &str) -> io::Result<Self> {
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(Self::from_str(&content, relative_path))
    }
}

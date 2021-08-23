use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{
    fs::{DirEntry, File},
    io::Read,
};

use crate::markdown::Markdown;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArticleMeta {
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub last_update: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Article {
    pub section: String,
    // filename may different with name, that's why we need disambiguation
    pub name: String,
    pub summary: String,
    #[serde(flatten)]
    pub metadata: ArticleMeta,
    // #[serde(skip_serializing)]
    pub content: Markdown,
}

impl Article {
    pub fn new(content: Markdown, meta: ArticleMeta, section: String) -> Self {
        Self {
            section,
            summary: content.summary(),
            name: content.name(),
            content,
            metadata: meta,
        }
    }

    pub fn load(entry: DirEntry) -> Self {
        let section = entry
            .path()
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let filename = entry
            .file_name()
            .into_string()
            .unwrap()
            .trim_end_matches(".md")
            .to_string();
        let mut file = File::open(entry.path()).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        let mut iter = content.split("---");
        iter.next();
        let meta_str = iter.next().unwrap();
        let meta = serde_yaml::from_str(meta_str).unwrap();
        let content = Markdown::new(filename, iter.next().unwrap());
        Self::new(content, meta, section)
    }
}

use crate::model::Article;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::DirEntry;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Section {
    pub name: String,
    pub articles: Vec<Article>,
}

impl Section {
    pub fn load(dir: DirEntry) -> Self {
        let name = dir
            .path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let articles = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|it| match it {
                Ok(it) if it.path().extension().unwrap() == "md" => Some(it),
                _ => None,
            })
            .map(Article::load)
            .collect();
        Self { name, articles }
    }
}

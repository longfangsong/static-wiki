use crate::markdown::Markdown;
use pulldown_cmark::Event;
use serde::{Deserialize, Serialize};
use std::{fs::File, io, io::Read};
use urlencoding::encode;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Header {
    category: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    aliases: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchIndex {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    aliases: Vec<String>,
    content_for_search: String,
    path: String,
}

pub struct ContentFile {
    path: String,
    header: Header,
    content: String,
}

impl Markdown for ContentFile {
    fn content(&self) -> &str {
        &self.content
    }

    fn path(&self) -> &str {
        &self.path
    }
}

impl ContentFile {
    pub fn search_index(&self) -> SearchIndex {
        let mut parser = self.parser();
        let name = parser.find(|it| matches!(it, Event::Text(_))).unwrap();
        let name = if let Event::Text(x) = name {
            x.into_string()
        } else {
            unreachable!()
        };
        let parser = self.parser();
        let content_for_search = parser
            .filter_map(|it| {
                if let Event::Text(x) = it {
                    Some(x)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("");
        SearchIndex {
            name,
            path: encode(&self.path),
            aliases: self.header.aliases.clone(),
            content_for_search,
            tags: self.header.tags.clone(),
        }
    }

    fn from_str(content: &str, path: &str) -> Self {
        let mut iter = content.splitn(3, "---");
        assert_eq!(iter.next(), Some(""));
        let (header_str, content) = (iter.next().unwrap(), iter.next().unwrap());
        let header = serde_yaml::from_str(header_str).unwrap();
        let mut path = path.strip_suffix(".md").unwrap().to_string();
        path.push_str(".htmlpart");
        Self {
            path,
            header,
            content: content.to_string(),
        }
    }
    pub fn from_file(mut file: File, relative_path: &str) -> io::Result<Self> {
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(Self::from_str(&content, relative_path))
    }
}

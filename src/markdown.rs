use lazy_static::lazy_static;
use pulldown_cmark::{html, Event, Options, Parser};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::io::Read;
use urlencoding::encode;
lazy_static! {
    static ref OPTIONS: Options = {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options
    };
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Header {
    category: String,
    tags: Vec<String>,
    #[serde(default)]
    aliases: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchIndex {
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
    content_for_search: String,
    path: String,
}

pub struct Markdown {
    path: String,
    header: Header,
    content: String,
}

impl Markdown {
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
        }
    }
    fn parser(&self) -> Parser {
        Parser::new_ext(&self.content, *OPTIONS)
    }
    pub fn html(&self) -> String {
        let mut html_output = String::new();
        html::push_html(&mut html_output, self.parser());
        html_output
    }
    pub fn from_str(content: &str, path: &str) -> Self {
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

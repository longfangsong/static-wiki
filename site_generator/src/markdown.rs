use lazy_static::lazy_static;
use pulldown_cmark::{html, Event, Options, Parser, Tag};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

lazy_static! {
    static ref OPTIONS: Options = {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options
    };
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Markdown {
    content: String,
}

impl Markdown {
    pub fn new(content: impl ToString) -> Self {
        Self {
            content: content.to_string(),
        }
    }
    pub fn load_from_path(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = File::open(path.as_ref())?;
        Ok(Self::load(file))
    }
    pub fn load(mut file: File) -> Self {
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        Self::new(content)
    }
    fn parser(&self) -> Parser {
        Parser::new_ext(&self.content, *OPTIONS)
    }
    pub fn html(&self) -> String {
        let mut html_output = String::new();
        html::push_html(&mut html_output, self.parser());
        html_output
    }
    pub fn content_without_title(&self) -> String {
        self.html().splitn(2, "</h1>").nth(1).unwrap().to_string()
    }
    pub fn summary(&self) -> String {
        let mut started = false;
        let mut result = String::new();
        for node in self.parser() {
            match node {
                Event::Start(Tag::Paragraph) => {
                    started = true;
                }
                Event::Text(t) => {
                    if started {
                        result += t.as_ref();
                    }
                }
                Event::Code(t) => {
                    if started {
                        result += t.as_ref();
                    }
                }
                Event::End(Tag::Paragraph) => {
                    if started {
                        return result;
                    }
                }
                _ => {}
            }
        }
        unreachable!();
    }
    pub fn name(&self) -> String {
        let mut started = false;
        for node in self.parser() {
            if let Event::Start(Tag::Heading(1)) = node {
                started = true;
            } else if let Event::Text(t) = node {
                if started {
                    return t.into_string();
                }
            }
        }
        unreachable!();
    }
}

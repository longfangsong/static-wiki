use lazy_static::lazy_static;
mod content_file;
mod index_file;
pub use content_file::{ContentFile, SearchIndex};
pub use index_file::IndexFile;
use pulldown_cmark::{html, Options, Parser};

lazy_static! {
    static ref OPTIONS: Options = {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options
    };
}

pub trait Markdown {
    fn content(&self) -> &str;
    fn parser(&self) -> Parser {
        Parser::new_ext(self.content(), *OPTIONS)
    }
    fn html(&self) -> String {
        let mut html_output = String::new();
        html::push_html(&mut html_output, self.parser());
        html_output
    }
    fn path(&self) -> &str;
}

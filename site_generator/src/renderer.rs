use crate::markdown::Markdown;
use crate::model::{Article, LanguageSite, Section, Site};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tera::{Context, Tera};

pub struct Renderer {
    tera: Tera,
}

impl Renderer {
    pub fn load_from_path(templates: &str) -> Self {
        let mut tera = match Tera::new(templates.trim_start_matches("./")) {
            Ok(t) => t,
            Err(e) => {
                panic!("Parsing error(s): {}", e);
            }
        };
        tera.autoescape_on(vec![]);
        Renderer { tera }
    }
}

impl Renderer {
    fn render_page(&self, context: &Context, markdown: &Markdown, path: impl AsRef<Path>) {
        let mut context = context.clone();
        context.insert("content", &markdown.html());
        let rendered = self.tera.render("page.html", &context).unwrap();
        let mut file = File::create(path).unwrap();
        write!(file, "{}", rendered).unwrap();
    }

    fn render_article(&self, context: &Context, article: &Article, path: impl AsRef<Path>) {
        let mut path = path.as_ref().join(&article.filename);
        path.set_extension("html");
        self.render_page(context, &article.content, path);
    }

    fn render_language_index(&self, context: &mut Context, path: impl AsRef<Path>) {
        let index = self.tera.render("index.html", &context).unwrap();
        let mut index_file = File::create(path.as_ref()).unwrap();
        write!(index_file, "{}", index).unwrap();
    }

    fn render_section(&self, context: &mut Context, section: &Section, path: impl AsRef<Path>) {
        fs::create_dir_all(&path).unwrap();
        let index = self.tera.render("subindex.html", &context).unwrap();
        let mut file = File::create(path.as_ref().join("index.html")).unwrap();
        write!(file, "{}", index).unwrap();
        for article in &section.articles {
            self.render_article(context, article, &path)
        }
    }
    fn render_language_site(
        &self,
        context: &mut Context,
        language_site: &LanguageSite,
        path: impl AsRef<Path>,
    ) {
        fs::create_dir_all(path.as_ref()).unwrap();
        context.insert("language_site", &language_site);
        context.insert("article_count", &language_site.article_count());
        context.insert("site_index", &language_site.collect_search_indexes());
        self.render_language_index(context, path.as_ref().join("index.html"));
        for (_, section) in &language_site.sections {
            context.insert("section", section);
            self.render_section(context, section, path.as_ref().join(&section.name));
        }
        self.render_page(
            context,
            &language_site.about,
            path.as_ref().join("about.html"),
        );
    }
    pub fn render_to(&self, site: Site, path: impl AsRef<Path>) {
        fs::remove_dir_all(path.as_ref()).unwrap_or(());
        fs::create_dir_all(path.as_ref()).unwrap();
        let mut context = Context::new();
        context.insert("site", &site);
        for (_, language_site) in site.language_sites {
            self.render_language_site(
                &mut context,
                &language_site,
                path.as_ref().join(&language_site.language),
            );
        }
    }
}

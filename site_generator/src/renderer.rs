use crate::markdown::Markdown;
use crate::model::{Article, DisambiguationSearchIndex, LanguageSite, SearchIndex, Section, Site};
use log::info;
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
        context.insert("name", &markdown.name());
        context.insert("content", &markdown.html());
        let rendered = self.tera.render("page.html", &context).unwrap();
        let mut file = File::create(path).unwrap();
        write!(file, "{}", rendered).unwrap();
    }
    fn render_disambiguation(
        &self,
        context: &mut Context,
        disambiguation: &DisambiguationSearchIndex,
        path: impl AsRef<Path>,
    ) {
        let mut context = context.clone();
        context.insert("disambiguation", disambiguation);
        let rendered = self.tera.render("disambiguation.html", &context).unwrap();
        let mut file = File::create(path).unwrap();
        write!(file, "{}", rendered).unwrap();
    }
    fn render_article(&self, context: &Context, article: &Article, path: impl AsRef<Path>) {
        let mut path = path.as_ref().join(&article.filename);
        path.set_extension("html");
        self.render_page(context, &article.content, path);
    }
    fn render_sitemap(&self, context: &mut Context, path: impl AsRef<Path>) {
        let sitemap = self.tera.render("sitemap.xml", &context).unwrap();
        let mut sitemap_file = File::create(path.as_ref()).unwrap();
        write!(sitemap_file, "{}", sitemap).unwrap();
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
        context.insert("now", &chrono::Utc::now());
        let articles: Vec<_> = language_site
            .sections
            .values()
            .map(|section| section.articles.iter())
            .flatten()
            .collect();
        context.insert("articles", &articles);
        self.render_sitemap(context, path.as_ref().join("sitemap.xml"));
        self.render_language_index(context, path.as_ref().join("index.html"));
        for (section_name, section) in &language_site.sections {
            info!("render section {} ...", section_name);
            context.insert("section", section);
            self.render_section(context, section, path.as_ref().join(&section.name));
        }
        fs::create_dir_all(path.as_ref().join("disambiguation")).unwrap();
        info!("render disambiguations ...");
        for disambiguation in language_site
            .collect_search_indexes()
            .iter()
            .filter_map(|it| {
                if let SearchIndex::Disambiguation(it) = it {
                    Some(it)
                } else {
                    None
                }
            })
        {
            let mut filepath = path
                .as_ref()
                .join("disambiguation")
                .join(&disambiguation.name);
            filepath.set_extension("html");
            self.render_disambiguation(context, disambiguation, filepath)
        }
        info!("render about ...");
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
        for (language, language_site) in site.language_sites.iter() {
            info!("Render {:?} site ...", language);
            self.render_language_site(
                &mut context,
                &language_site,
                path.as_ref().join(&language_site.language),
            );
        }
        let mut index = File::create(path.as_ref().join("index.html")).unwrap();
        let primary_language = if site.language_sites.contains_key("zh") {
            "zh"
        } else {
            site.language_sites.keys().next().unwrap()
        };
        write!(
            index,
            r#"<!DOCTYPE html>
<meta charset="utf-8">
<title>Redirecting to {}/{}/index.html</title>
<meta http-equiv="refresh" content="0; URL={}/{}/index.html">
<link rel="canonical" href="{}/{}/index.html">"#,
            site.config.public_url,
            primary_language,
            site.config.public_url,
            primary_language,
            site.config.public_url,
            primary_language
        )
        .unwrap();
    }
}

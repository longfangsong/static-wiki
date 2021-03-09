use crate::markdown::Markdown;
use derive_more::From;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::{DirEntry, File};
use std::io::Read;
use std::iter::Iterator;
use std::path::Path;
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArticleMeta {
    category: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    aliases: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Article {
    section: String,
    // filename may different with name, that's why we need disambiguation
    pub filename: String,
    pub name: String,
    pub summary: String,
    #[serde(flatten)]
    pub metadata: ArticleMeta,
    #[serde(skip_serializing)]
    pub content: Markdown,
}

impl Article {
    pub fn new(filename: String, content: Markdown, meta: ArticleMeta, section: String) -> Self {
        Self {
            section,
            filename,
            summary: content.summary(),
            name: content.name(),
            content,
            metadata: meta,
        }
    }

    pub fn load(entry: DirEntry, section: String) -> Self {
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
        let content = Markdown::new(iter.next().unwrap());
        Self::new(filename, content, meta, section)
    }
}

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
            .map(|it| Article::load(it, name.clone()))
            .collect();
        Self { name, articles }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LanguageSite {
    pub language: String,
    pub sections: HashMap<String, Section>,
    pub disambiguation: HashMap<String, Vec<Article>>,
    pub about: Markdown,
    pub translation: toml::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArticleSearchIndex {
    #[serde(default)]
    section: String,
    #[serde(default)]
    category: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    filename: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DisambiguationSearchIndex {
    #[serde(default)]
    name: String,
    #[serde(default)]
    articles: Vec<ArticleSearchIndex>,
}

#[derive(Debug, Clone, Deserialize, Serialize, From)]
#[serde(untagged)]
pub enum SearchIndex {
    Article(ArticleSearchIndex),
    Disambiguation(DisambiguationSearchIndex),
}

impl Into<ArticleSearchIndex> for Article {
    fn into(self) -> ArticleSearchIndex {
        ArticleSearchIndex {
            section: self.section,
            category: self.metadata.category,
            tags: self.metadata.tags,
            aliases: self.metadata.aliases,
            summary: self.summary,
            name: self.name,
            filename: self.filename,
        }
    }
}

impl Into<DisambiguationSearchIndex> for Vec<Article> {
    fn into(self) -> DisambiguationSearchIndex {
        assert_ne!(self.len(), 0);
        DisambiguationSearchIndex {
            name: self[0].name.clone(),
            articles: self.into_iter().map(Article::into).collect(),
        }
    }
}

impl LanguageSite {
    fn new(
        language: String,
        sections_vec: Vec<Section>,
        disambiguation: HashMap<String, Vec<Article>>,
        about: Markdown,
        translation: toml::Value,
    ) -> Self {
        let mut sections = HashMap::new();
        for section in sections_vec {
            sections.insert(section.name.clone(), section);
        }
        Self {
            language,
            sections,
            disambiguation,
            about,
            translation,
        }
    }

    fn collect_name_articles_map(
        sections: impl Iterator<Item = Section>,
    ) -> HashMap<String, Vec<Article>> {
        let articles: Vec<_> = sections
            .map(|section| section.articles.into_iter())
            .flatten()
            .collect();
        let mut result = HashMap::new();
        for article in articles {
            result
                .entry(article.name.clone())
                .or_insert_with(Vec::new)
                .push(article.clone())
        }
        result
    }

    fn collect_disambiguation(sections: &[Section]) -> HashMap<String, Vec<Article>> {
        Self::collect_name_articles_map(sections.iter().cloned())
            .iter()
            .filter(|(_, articles)| articles.len() > 1)
            .map(|(name, article)| (name.clone(), article.clone()))
            .collect()
    }

    pub fn collect_search_indexes(&self) -> Vec<SearchIndex> {
        let (disambiguation_pages, simple_pages): (Vec<_>, Vec<_>) =
            Self::collect_name_articles_map(self.sections.values().cloned())
                .values()
                .cloned()
                .partition(|articles| articles.len() > 1);
        let simple_pages = simple_pages
            .into_iter()
            .filter_map(|it| it.first().cloned());
        disambiguation_pages
            .into_iter()
            .map(|it| it.into())
            .map(|it: DisambiguationSearchIndex| it.into())
            .chain(
                simple_pages
                    .map(|it| it.into())
                    .map(|it: ArticleSearchIndex| it.into()),
            )
            .collect()
    }

    fn load(dir: fs::DirEntry) -> Self {
        let sections_vec: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|it| it.ok())
            .filter(|it| it.metadata().unwrap().is_dir())
            .map(Section::load)
            .collect();
        let disambiguation = Self::collect_disambiguation(&sections_vec);
        let about_content = Markdown::load_from_path(dir.path().join("about.md")).unwrap();
        let mut translation_file = File::open(dir.path().join("translation.toml")).unwrap();
        let mut translation_content = String::new();
        translation_file
            .read_to_string(&mut translation_content)
            .unwrap();
        let translation: toml::Value = toml::from_str(&translation_content).unwrap();

        Self::new(
            dir.file_name().into_string().unwrap(),
            sections_vec,
            disambiguation,
            about_content,
            translation,
        )
    }

    pub fn article_count(&self) -> usize {
        self.sections.values().map(|it| it.articles.len()).sum()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SiteConfig {
    pub title: String,
    pub public_url: String,
    pub description: String,
}

impl SiteConfig {
    pub fn load(mut file: File) -> Self {
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        toml::from_str(&content).unwrap()
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Self {
        let file = File::open(path.as_ref()).unwrap();
        Self::load(file)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Site {
    #[serde(flatten)]
    pub config: SiteConfig,
    pub language_sites: HashMap<String, LanguageSite>,
}

impl Site {
    fn new(config: SiteConfig, language_site_vec: Vec<LanguageSite>) -> Self {
        let mut language_sites = HashMap::new();
        for language in language_site_vec {
            language_sites.insert(language.language.clone(), language);
        }
        Self {
            config,
            language_sites,
        }
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Self {
        let config = SiteConfig::load_from_path(path.as_ref().join("site.toml"));
        let language_site_vec = fs::read_dir(path)
            .unwrap()
            .filter_map(|it| it.ok())
            .filter(|it| it.metadata().unwrap().is_dir())
            .map(LanguageSite::load)
            .collect();
        Self::new(config, language_site_vec)
    }
}

use crate::markdown::Markdown;
use crate::model::{Article, ArticleSearchIndex, DisambiguationSearchIndex, SearchIndex, Section};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Disambiguation {
    pub name: String,
    pub articles: Vec<Article>,
}

impl TryFrom<Vec<Article>> for Disambiguation {
    type Error = ();

    fn try_from(articles: Vec<Article>) -> Result<Self, Self::Error> {
        if articles.len() <= 1 {
            Err(())
        } else {
            let name = articles[0].name.clone();
            for article in &articles {
                if article.name != name {
                    return Err(());
                }
            }
            Ok(Self { name, articles })
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LanguageSite {
    pub language: String,
    pub sections: HashMap<String, Section>,
    pub disambiguation: Vec<Disambiguation>,
    pub about: Markdown,
    pub translation: toml::Value,
}

impl LanguageSite {
    fn new(
        language: String,
        sections_vec: Vec<Section>,
        disambiguation: Vec<Vec<Article>>,
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
            disambiguation: disambiguation
                .into_iter()
                .map(|x| Disambiguation::try_from(x).unwrap())
                .collect(),
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

    fn collect_disambiguation(sections: &[Section]) -> Vec<Vec<Article>> {
        Self::collect_name_articles_map(sections.iter().cloned())
            .iter()
            .filter(|(_, articles)| articles.len() > 1)
            .map(|(_name, article)| article.clone())
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

    pub(crate) fn load(dir: fs::DirEntry) -> Self {
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
        self.sections.iter().map(|(_, it)| it.articles.len()).sum()
    }
}

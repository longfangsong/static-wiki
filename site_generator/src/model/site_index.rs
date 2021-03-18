use crate::model::Article;
use derive_more::From;
use serde::{Deserialize, Serialize};
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
    pub name: String,
    #[serde(default)]
    pub articles: Vec<ArticleSearchIndex>,
}

#[derive(Debug, Clone, Deserialize, Serialize, From)]
#[serde(untagged)]
pub enum SearchIndex {
    Article(ArticleSearchIndex),
    Disambiguation(DisambiguationSearchIndex),
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

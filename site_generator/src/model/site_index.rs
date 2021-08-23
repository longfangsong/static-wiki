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

impl From<Vec<Article>> for DisambiguationSearchIndex {
    fn from(articles: Vec<Article>) -> Self {
        assert_ne!(articles.len(), 0);
        Self {
            name: articles[0].name.clone(),
            articles: articles.into_iter().map(Article::into).collect(),
        }
    }
}

impl From<Article> for ArticleSearchIndex {
    fn from(article: Article) -> Self {
        Self {
            section: article.section,
            category: article.metadata.category,
            tags: article.metadata.tags,
            aliases: article.metadata.aliases,
            summary: article.summary,
            name: article.name,
            filename: article.content.filename,
        }
    }
}

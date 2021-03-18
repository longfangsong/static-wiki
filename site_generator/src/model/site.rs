use crate::model::LanguageSite;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;

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

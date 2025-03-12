use std::{collections::HashMap, path::PathBuf};
use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Clone)]
pub enum Layout {
    Index,
    Article
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Cfg {
    pub title: String,
    pub site_folder: PathBuf, // Relative path to the vault
    pub site_css: Option<String>, // Optional CSS overrides
}

impl Into<Cfg> for PathBuf {
    fn into(self) -> Cfg {
        // Canonicalize path to handle relative and home paths properly
        let path = self.canonicalize().expect("Failed to canonicalize path");
        let content = std::fs::read_to_string(&path).expect("Failed to read config file");
        toml::from_str(&content).expect("Failed to parse config file")
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Site {
    pub cfg: Cfg,
    pub site_notes: Vec<crate::obsidian::Note>,
    pub linked_notes: HashMap<String, crate::obsidian::Note>,
    pub embedded_notes: HashMap<String, crate::obsidian::Note>,
}

impl Site {
    pub fn from_vault(vault: &crate::obsidian::Vault, cfg: Cfg) -> Self {
        let full_path = vault.path.join(&cfg.site_folder);
        let site_notes = vault.notes.iter()
            .filter(|note| note.path.starts_with(&full_path))
            .cloned()
            .collect();
        let linked_notes = vault.linked_notes(&site_notes);
        let embedded_notes = vault.embedded_notes(&site_notes);
        Site { cfg, site_notes, linked_notes, embedded_notes }
    }
}
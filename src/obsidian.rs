use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

type ObsidianMd = String;

#[derive(Deserialize, Serialize, Clone)]
pub struct Metadata {
    pub date: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub aliases:  Vec<String>,
    pub layout: Option<crate::site::Layout>,
    pub permalink: Option<String>,
    #[serde(default)]
    pub private: bool // Determines whether the note will be published
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Note {
    pub metadata: Metadata, // Frontmatter metadata
    pub path: PathBuf, // Relative path to the vault with extension
    pub slug: String, // Title of the note as defined by osbidian
    pub title: String, // First heading of a doc; if not found, defaults to the filename without extension
    pub content: ObsidianMd, // Obsidian markdown content without the frontmatter
}

impl Note {
    fn extract_frontmatter(raw_content: &str) -> (String, String) {
        // return a tuple of frontmatter yaml and the markdown content
        let mut lines = raw_content.lines();
        let mut frontmatter = String::new();
        let mut content = String::new();
        if let Some(line) = lines.next() {
            if line == "---" {
                while let Some(line) = lines.next() {
                    if line == "---" {
                        break;
                    }
                    frontmatter.push_str(line);
                    frontmatter.push('\n');
                }
            }
            content.push_str(line);
            content.push('\n');
            for line in lines {
                content.push_str(line);
                content.push('\n');
            }
        } 
        (frontmatter, content)
    }

    pub fn extract_title(content: &ObsidianMd) -> ObsidianMd {
        content
            .lines()
            .find(|line| line.starts_with("# "))
            .map(|line| line[2..].trim().to_string())
            .unwrap_or_else(|| "Untitled".to_string())
    }

    fn linked_notes(&self, vault: &Vault) -> HashMap<String, Note> {
        // Return a hashmap of reference substring to note; denoted by [[Note Title]] or [[Note Title|Alias]]
        let mut linked_notes = HashMap::new();
        for line in self.content.lines() {
            let mut words = line.split_whitespace();
            while let Some(word) = words.next() {
                if word.starts_with("[[") {
                    let linked = word.trim_start_matches("[[").trim_end_matches("]]");
                    let referenced_title = if linked.contains('|') {
                        linked.split('|').next().unwrap()
                    } else {
                        linked
                    };
                    match vault.find(referenced_title) {
                        Ok(note) => {
                            linked_notes.insert(referenced_title.to_string(), note);
                        }
                        Err(_) => {
                            eprintln!("Note not found: {}", referenced_title);
                        }
                    }

                }
            }
        }
        linked_notes

    }

    fn embedded_notes(&self, vault: &Vault) -> HashMap<String, Note> {
        // Return a hashmap of reference substring to note; denoted by ![[Note Title]]
        let mut embedded_notes = HashMap::new();
        for line in self.content.lines() {
            let mut words = line.split_whitespace();
            while let Some(word) = words.next() {
                if word.starts_with("![[") {
                    let embedded = word.trim_start_matches("![[[").trim_end_matches("]]");
                    let referenced_title = if embedded.contains('|') {
                        embedded.split('|').next().unwrap()
                    } else {
                        embedded
                    };
                    match vault.find(referenced_title) {
                        Ok(note) => {
                            embedded_notes.insert(referenced_title.to_string(), note);
                        }
                        Err(_) => {
                            eprintln!("Note not found: {}", referenced_title);
                        }
                    }
                }
            }
        }
        embedded_notes
    }
}

impl Into<Note> for PathBuf {
    fn into(self) -> Note {
        let path = self;
        let content = std::fs::read_to_string(&path).unwrap();
        let (frontmatter, content) = Note::extract_frontmatter(&content);
        let metadata: Metadata = serde_yaml::from_str(&frontmatter).unwrap();
        let slug = path.file_stem().unwrap().to_string_lossy().to_string();
        let title = Note::extract_title(&content);
        Note {
            metadata,
            path,
            slug,
            title,
            content
        }
    }
}

pub struct Vault {
    pub path: PathBuf,
    pub notes: Vec<Note>
}

impl From<PathBuf> for Vault {
    fn from(path: PathBuf) -> Self {
        // walks the entire path and returns a vector of notes, process only .md files
        let mut notes = Vec::new();
        for entry in WalkDir::new(&path)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| {
            e.file_name()
                .to_str()
                .map(|s| !s.starts_with('.') && !s.starts_with('_'))
                .unwrap_or(true)
            })
            .filter_map(|e| e.ok()) {
            let path = entry.path().to_path_buf();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            notes.push(path.into());
            }
        }
        Vault {
            path,
            notes
        }
    }
}

impl Vault {
    fn find(&self, note_reference: &str) -> Result<Note> {
        // Find a note by its title or alias
        let note = self.notes.iter().find(|n| {
            n.slug == note_reference || n.metadata.aliases.contains(&note_reference.to_string())
        });
        match note {
            Some(note) => Ok(note.clone()),
            None => Err(anyhow::anyhow!("Note not found"))
        }
    }
}


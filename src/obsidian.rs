use std::collections::HashMap;
use std::path::PathBuf;

type ObsidianMd = String;

pub struct Vault {
    pub path: PathBuf,
    pub files: HashMap<PathBuf, ObsidianMd>,
}

impl From<PathBuf> for Vault {
    fn from(path: PathBuf) -> Self {
        let mut files = HashMap::new();
        fn visit_dir(dir: &PathBuf, files: &mut HashMap<PathBuf, ObsidianMd>) {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                files.insert(path.clone(), content);
                            }
                        } else if path.is_dir() {
                            visit_dir(&path, files);
                        }
                    }
                }
            }
        }

        visit_dir(&path, &mut files);
        Vault { path, files }
    }
}

pub fn extract_title(content: &ObsidianMd) -> ObsidianMd {
    content
        .lines()
        .find(|line| line.starts_with("# "))
        .map(|line| line[2..].trim().to_string())
        .unwrap_or_else(|| "Untitled".to_string())
}

pub fn process_embeds(content: &ObsidianMd, vault: &Vault) -> ObsidianMd {
    // Match Obsidian embeds like ![[Some Note]]
    let embed_regex = regex::Regex::new(r"!\[\[(.*?)\]\]").unwrap();

    embed_regex
        .replace_all(content, |caps: &regex::Captures| {
            let note_name = &caps[1];
            let note_path = vault.files.keys().find(|path| {
                let file_name = path.file_stem().unwrap_or_default().to_string_lossy();
                file_name == note_name
            });

            if let Some(path) = note_path {
                if let Some(embed_content) = vault.files.get(path) {
                    format!("\n\n{}\n\n", embed_content)
                } else {
                    format!("![[{}]] (not found)", note_name)
                }
            } else {
                format!("![[{}]] (not found)", note_name)
            }
        })
        .to_string()
}

pub fn process_links(content: &ObsidianMd) -> ObsidianMd {
    // Match Obsidian internal links like [[Some Note]] or [[Some Note|Custom Text]]
    let link_regex = regex::Regex::new(r"\[\[(.*?)(?:\|(.*?))?\]\]").unwrap();

    link_regex
        .replace_all(content, |caps: &regex::Captures| {
            let note_name = &caps[1];
            let display_text = caps.get(2).map_or(note_name, |m| m.as_str());
            let url = note_name.replace(" ", "-").to_lowercase() + ".html";

            format!("[{}]({})", display_text, url)
        })
        .to_string()
}

pub fn remove_frontmatter(content: &ObsidianMd) -> ObsidianMd {
    let frontmatter_regex = regex::Regex::new(r"---\n(.*\n)*---\n").unwrap();
    frontmatter_regex.replace(content, "").to_string()
}

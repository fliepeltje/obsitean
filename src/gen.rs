use crate::obsidian;
use crate::site::{Page, SiteCfg};

fn gen_links(sitecfg: &SiteCfg, vault: &obsidian::Vault) -> Vec<(String, String)> {
    let mut links = Vec::new();
    let target_folder = vault.path.join(&sitecfg.vault_folder);
    for (path, _) in &vault.files {
        if path.starts_with(&target_folder) {
            let rel_path = path.strip_prefix(&target_folder).unwrap();
            let rel_path = rel_path.with_extension("").with_extension("html");
            let url = rel_path.to_string_lossy().to_string();
            let title = obsidian::extract_title(&vault.files[path]);
            links.push((title, url));
        }
    }
    // Sort links: "index.html" first, then alphabetically by title
    links.sort_by(|(title_a, url_a), (title_b, url_b)| {
        if url_a == "index.html" {
            std::cmp::Ordering::Less
        } else if url_b == "index.html" {
            std::cmp::Ordering::Greater
        } else {
            title_a.to_lowercase().cmp(&title_b.to_lowercase())
        }
    });
    links
}

pub fn gen_pages(site_cfg: &SiteCfg, vault: &obsidian::Vault) -> Vec<Page> {
    let mut pages = Vec::new();
    let target_folder = vault.path.join(&site_cfg.vault_folder);
    let links = gen_links(site_cfg, vault);
    for (path, content) in &vault.files {
        // Check if the path is inside the specified relative path in site_cfg

        if path.starts_with(&target_folder) {
            let title = obsidian::extract_title(content);
            let content = obsidian::process_embeds(content, vault);
            let content = obsidian::process_links(&content);
            let content = obsidian::remove_frontmatter(&content);
            let rel_path = path.strip_prefix(&target_folder).unwrap();
            let page = Page {
                site_title: site_cfg.title.clone(),
                page_title: title,
                links: links.clone(),
                current_page: rel_path.to_string_lossy().to_string(),
                content: content.to_owned(),
            };
            pages.push(page);
        }
    }
    pages
}

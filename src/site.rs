use askama::Template;
use std::path::PathBuf;

pub static CSS: &'static str = include_str!("../static/css/style.css");

pub struct SiteCfg {
    pub title: String,
    pub vault_folder: PathBuf, // Relative path to the Obsidian vault
}

#[derive(Template)]
#[template(path = "page.jinja2")]
pub struct Page {
    pub site_title: String,
    pub page_title: String,
    pub links: Vec<(String, String)>,
    pub current_page: String,
    pub content: String,
}

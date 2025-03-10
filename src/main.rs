use askama::Template;
use clap::Parser;
use std::path::PathBuf;
mod gen;
mod obsidian;
mod site;

#[derive(Parser)]
#[command(author, version, about = "Static site generator for Obsidian vaults")]
struct CliOpts {
    /// Path to the Obsidian vault
    #[arg(short, long)]
    vault_path: PathBuf,

    /// Title for the generated site
    #[arg(short, long)]
    site_title: String,

    /// Target folder within vault to process
    #[arg(short, long)]
    target: PathBuf,

    /// Output directory for generated site
    #[arg(short, long)]
    out_path: PathBuf,
}

fn main() {
    // Create a sample Page instance
    let opts = CliOpts::parse();
    let vault = obsidian::Vault::from(opts.vault_path);
    let site_cfg = site::SiteCfg {
        title: opts.site_title.to_string(),
        vault_folder: opts.target.into(),
    };
    let pages = gen::gen_pages(&site_cfg, &vault);
    // Create the output directory if it doesn't exist
    let out_dir = opts.out_path;
    std::fs::create_dir_all(&out_dir).expect("Failed to create output directory");

    // Write each page to the output directory
    for page in pages {
        let file_path = out_dir.join(&page.current_page).with_extension("html");

        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create parent directories");
        }

        // Write the page content to file
        let rendered_html = page.render().unwrap();
        std::fs::write(file_path, rendered_html).expect("Failed to write page to file");
    }
    let css = site::CSS;
    let css_path = out_dir.join("style.css");
    std::fs::write(css_path, css).expect("Failed to write CSS to file");
}

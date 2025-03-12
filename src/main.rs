use clap::{Parser, Subcommand};
use std::path::PathBuf;
mod obsidian;
mod site;
mod templates;
mod server;

#[derive(Parser)]
#[command(author, version, about = "CLI for running obsidian websites")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Build the site")]
    Build {
        #[clap(long, help = "Path to the obsidian vault")]
        vault: PathBuf,
        #[clap(long, help = "Path to the site config file")]
        site_cfg: PathBuf,
        #[clap(long, help = "Path to the output file")]
        output: PathBuf,
    },
    #[command(about = "Serve the site")]
    Serve {
        site_data: PathBuf,
    }

}

#[tokio::main]
async fn main() {
    // Create a sample Page instance
    let opts = Cli::parse();
    match opts.command {
        Command::Build { vault, site_cfg, output } => {
            let vault = obsidian::Vault::from(vault);
            let site_cfg = site_cfg.into();
            let site = site::Site::from_vault(&vault, site_cfg);
            let json = serde_json::to_string_pretty(&site).expect("Failed to serialize site data");
            std::fs::write(&output, json).expect("Failed to write site data to file");
        },
        Command::Serve { site_data } => {
            let site_data = std::fs::read_to_string(&site_data).expect("Failed to read site data");
            let site: site::Site = serde_json::from_str(&site_data).expect("Failed to parse site data");
            let server = server::wiki_router(site);
            let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
            axum::serve(listener, server).await.unwrap();
        }
    }
}

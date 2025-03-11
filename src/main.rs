use clap::{Parser, Subcommand};
use std::path::PathBuf;
mod obsidian;
mod site;

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

}


fn main() {
    // Create a sample Page instance
    let opts = Cli::parse();
    match opts.command {
        Command::Build { vault, site_cfg, output } => {
            let vault = obsidian::Vault::from(vault);
            let site_cfg = site_cfg.into();
            let site = site::Site::from_vault(&vault, site_cfg);
            let json = serde_json::to_string_pretty(&site).expect("Failed to serialize site data");
            std::fs::write(&output, json).expect("Failed to write site data to file");
        }
    }
}

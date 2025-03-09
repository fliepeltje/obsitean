use anyhow::Result;
use askama::Template;
use clap::Parser;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser as MarkdownParser, Tag};
use regex::Regex;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the Obsidian vault root directory
    #[arg(short, long)]
    vault_dir: PathBuf,

    /// Subdirectory within the vault to process
    #[arg(short, long)]
    process_dir: String,

    /// Output directory for generated HTML files
    #[arg(short, long)]
    output_dir: PathBuf,
}

struct MarkdownFile {
    path: PathBuf,
    content: String,
    title: String,
    relative_path: PathBuf,
}

struct Heading {
    id: String,
    text: String,
    level: usize,
}

struct ArticleLink {
    title: String,
    url: String,
}

#[derive(Template)]
#[template(path = "page.html")]
struct PageTemplate<'a> {
    title: &'a str,
    content: &'a str,
    articles: &'a [ArticleLink],
    headings: &'a [Heading],
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Full path to the directory to process
    let process_path = args.vault_dir.join(&args.process_dir);
    if !process_path.exists() {
        return Err(anyhow::anyhow!("Process directory does not exist: {:?}", process_path));
    }
    
    // Create output directory if it doesn't exist
    fs::create_dir_all(&args.output_dir)?;
    
    // Collect all markdown files
    let markdown_files = collect_markdown_files(&process_path)?;
    
    // Create a lookup map for resolving file references
    let file_map = create_file_map(&markdown_files);
    
    // Generate article links for navigation
    let articles = generate_article_links(&markdown_files);
    
    // Process each markdown file
    for file in &markdown_files {
        // Process the markdown content
        let processed_content = process_markdown_content(&file.content, &file_map)?;
        
        // We need to remove the first heading from the content since it's used as the page title
        let content_without_first_heading = remove_first_heading(&processed_content);
        
        // Extract headings for the right-side navigation (after removing the first heading)
        let headings = extract_headings(&content_without_first_heading);
        
        // Convert markdown to HTML
        let html_content = markdown_to_html(&content_without_first_heading);
        
        // Render the template
        let template = PageTemplate {
            title: &file.title,
            content: &html_content,
            articles: &articles,
            headings: &headings,
        };
        
        let html = template.render()?;
        
        // Create output path
        let mut output_path = args.output_dir.join(&file.relative_path);
        output_path.set_extension("html");
        
        // Ensure parent directories exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Write HTML to file
        let mut output_file = File::create(&output_path)?;
        output_file.write_all(html.as_bytes())?;
        
        println!("Generated: {:?}", output_path);
    }
    
    println!("Static site generation complete!");
    Ok(())
}

fn collect_markdown_files(dir: &Path) -> Result<Vec<MarkdownFile>> {
    let mut files = Vec::new();
    
    for entry in WalkDir::new(dir) {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            let mut file = File::open(path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            
            // Extract title from first heading, or fall back to filename
            let title = extract_first_heading(&content).unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled")
                    .replace('_', " ")
            });
            
            // Calculate path relative to the processing directory
            let relative_path = path.strip_prefix(dir)?.to_path_buf();
            
            files.push(MarkdownFile {
                path: path.to_path_buf(),
                content,
                title,
                relative_path,
            });
        }
    }
    
    Ok(files)
}

// New function to extract the first heading from markdown content
fn extract_first_heading(content: &str) -> Option<String> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    
    let parser = MarkdownParser::new_ext(content, options);
    let mut current_heading: Option<String> = None;
    let mut in_heading = false;
    
    for event in parser {
        match event {
            Event::Start(Tag::Heading(level, _, _)) if level == HeadingLevel::H1 => {
                in_heading = true;
                current_heading = Some(String::new());
            },
            Event::Text(text) => {
                if in_heading {
                    if let Some(ref mut heading) = current_heading {
                        heading.push_str(&text);
                    }
                }
            },
            Event::End(Tag::Heading(level, _, _)) if level == HeadingLevel::H1 => {
                break; // Exit after finding first heading
            },
            _ => {}
        }
    }
    
    current_heading
}

fn create_file_map(files: &[MarkdownFile]) -> HashMap<String, &MarkdownFile> {
    let mut map = HashMap::new();
    
    for file in files {
        if let Some(stem) = file.path.file_stem().and_then(|s| s.to_str()) {
            map.insert(stem.to_string(), file);
        }
    }
    
    map
}

fn generate_article_links(files: &[MarkdownFile]) -> Vec<ArticleLink> {
    files.iter()
        .map(|file| {
            let mut url = file.relative_path.to_string_lossy().to_string();
            if let Some(pos) = url.rfind('.') {
                url.truncate(pos);
            }
            url.push_str(".html");
            
            ArticleLink {
                title: file.title.clone(),
                url,
            }
        })
        .collect()
}

fn process_markdown_content(
    content: &str,
    file_map: &HashMap<String, &MarkdownFile>,
) -> Result<String> {
    let embed_regex = Regex::new(r"!\[\[(.*?)\]\]")?;
    let link_regex = Regex::new(r"\[\[(.*?)\]\]")?;
    
    // Process embeddings
    let mut result = content.to_string();
    result = embed_regex.replace_all(&result, |caps: &regex::Captures| {
        let file_name = &caps[1];
        if let Some(file) = file_map.get(file_name) {
            // Include the content of the referenced file
            file.content.clone()
        } else {
            format!("<!-- Embedding not found: {} -->", file_name)
        }
    }).to_string();
    
    // Process internal links
    result = link_regex.replace_all(&result, |caps: &regex::Captures| {
        let link_text = &caps[1];
        let display_text = link_text;
        
        // Check if this is a reference to another file in our collection
        if let Some(file) = file_map.get(link_text) {
            // Use the title from the file's first heading if available
            format!("[{}]({}.html)", file.title, link_text)
        } else {
            // Fall back to original behavior
            format!("[{}]({}.html)", display_text, link_text)
        }
    }).to_string();
    
    Ok(result)
}

fn extract_headings(content: &str) -> Vec<Heading> {
    let mut headings = Vec::new();
    let mut options = Options::empty();
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    
    let parser = MarkdownParser::new_ext(content, options);
    let mut current_heading: Option<(String, usize)> = None;
    
    for event in parser {
        match event {
            Event::Start(Tag::Heading(level, _, _)) => {
                current_heading = Some((String::new(), level as usize));
            },
            Event::Text(text) => {
                if let Some((ref mut heading_text, _)) = current_heading {
                    heading_text.push_str(&text);
                }
            },
            Event::End(Tag::Heading(..)) => {
                if let Some((text, level)) = current_heading.take() {
                    // Create ID from heading text (simplified slug)
                    let id = text.to_lowercase()
                        .chars()
                        .map(|c| if c.is_alphanumeric() { c } else if c.is_whitespace() { '-' } else { '-' })
                        .collect::<String>()
                        .replace("--", "-");
                    
                    headings.push(Heading { id, text, level });
                }
            },
            _ => {}
        }
    }
    
    headings
}

fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    
    let parser = MarkdownParser::new_ext(markdown, options);
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);
    
    html_output
}

// New function to remove the first heading from content
fn remove_first_heading(content: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    
    let parser = MarkdownParser::new_ext(content, options);
    let mut events = Vec::new();
    let mut skip_next_text = false;
    let mut found_first_heading = false;
    
    for event in parser {
        match event {
            Event::Start(Tag::Heading(level, _, _)) if level == HeadingLevel::H1 && !found_first_heading => {
                found_first_heading = true;
                skip_next_text = true;
                continue;  // Skip this event
            },
            Event::Text(_) if skip_next_text => {
                skip_next_text = false;
                continue;  // Skip this event
            },
            Event::End(Tag::Heading(level, _, _)) if level == HeadingLevel::H1 && found_first_heading => {
                found_first_heading = false;  // Only skip the first heading
                continue;  // Skip this event
            },
            _ => {}
        }
        events.push(event);
    }
    
    // Convert events back to markdown (simplified approach)
    let mut output = String::new();
    pulldown_cmark::html::push_html(&mut output, events.into_iter());
    
    output
}

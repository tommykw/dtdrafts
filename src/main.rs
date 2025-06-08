use clap::Parser;
use anyhow::{Result, Context};
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "dtdrafts")]
#[command(about = "Search your dev.to draft articles")]
#[command(version = "0.1.0")]
struct Cli {
    /// Search query
    #[arg(short, long)]
    query: Option<String>,

    /// Set dev.to API key
    #[arg(long)]
    set_api_key: Option<String>,

    /// Force refresh cached articles
    #[arg(short, long)]
    refresh: bool,

    /// Show all drafts without filtering
    #[arg(short, long)]
    all: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct Article {
    id: u64,
    title: String,
    description: Option<String>,
    body_markdown: Option<String>,
    url: String,
    canonical_url: Option<String>,
    url_with_preview: Option<String>,
    published: bool,
    created_at: Option<String>,
    updated_at: Option<String>,
    tags: Option<Vec<String>>,
    slug: String,
    user: ArticleUser,
}

#[derive(Debug, Deserialize, Serialize)]
struct ArticleUser {
    username: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    api_key: String,
}

struct DevToClient {
    client: reqwest::Client,
    api_key: String,
}

impl DevToClient {
    fn new(api_key: String) -> Self {
        let client = reqwest::Client::new();
        Self { client, api_key }
    }

    async fn get_my_articles(&self) -> Result<Vec<Article>> {
        let mut all_articles = Vec::new();
        let mut page = 1;
        let per_page = 1000;
        let base_url = "https://dev.to/api";

        loop {
            let url = format!("{}/articles/me/unpublished?page={}&per_page={}", base_url, page, per_page);
            let response = self
                .client
                .get(&url)
                .header("api-key", &self.api_key)
                .header("User-Agent", "dtdrafts/0.1.0")
                .send()
                .await
                .context("Failed to fetch articles from dev.to API")?;

            if !response.status().is_success() {
                return Err(anyhow::anyhow!(
                    "API request failed with status: {}. Please check your API key.",
                    response.status()
                ));
            }

            let text = response.text().await?;
            let articles: Vec<Article> = serde_json::from_str(&text)
                .context("Failed to parse JSON response")?;

            let count = articles.len();
            if count == 0 {
                break;
            }
            all_articles.extend(articles);
            println!("Page {}: Fetched {} articles so far...", page, all_articles.len());
            page += 1;
            tokio::time::sleep(std::time::Duration::from_secs(1)).await; // rate limit対策
        }

        println!("Done! Total {} articles fetched.", all_articles.len());
        Ok(all_articles)
    }
}

fn get_config_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let config_dir = home_dir.join(".dtdrafts");
    Ok(config_dir)
}

fn get_config_file() -> Result<PathBuf> {
    let mut config_file = get_config_dir()?;
    config_file.push("config.json");
    Ok(config_file)
}

fn get_cache_file() -> Result<PathBuf> {
    let mut cache_file = get_config_dir()?;
    cache_file.push("articles_cache.json");
    Ok(cache_file)
}

fn save_config(config: &Config) -> Result<()> {
    let config_dir = get_config_dir()?;
    fs::create_dir_all(&config_dir)?;
    
    let config_file = get_config_file()?;
    let config_json = serde_json::to_string_pretty(config)?;
    fs::write(config_file, config_json)?;
    
    Ok(())
}

fn load_config() -> Result<Config> {
    let config_file = get_config_file()?;
    
    if !config_file.exists() {
        return Err(anyhow::anyhow!(
            "No API key found. Please set it first with: dtdrafts --set-api-key YOUR_API_KEY"
        ));
    }

    let config_content = fs::read_to_string(config_file)?;
    let config: Config = serde_json::from_str(&config_content)?;
    
    Ok(config)
}

fn save_articles_cache(articles: &[Article]) -> Result<()> {
    let config_dir = get_config_dir()?;
    fs::create_dir_all(&config_dir)?;
    
    let cache_file = get_cache_file()?;
    let cache_json = serde_json::to_string_pretty(articles)?;
    fs::write(cache_file, cache_json)?;
    
    Ok(())
}

fn load_articles_cache() -> Result<Vec<Article>> {
    let cache_file = get_cache_file()?;
    
    if !cache_file.exists() {
        return Ok(Vec::new());
    }

    let cache_content = fs::read_to_string(cache_file)?;
    let articles: Vec<Article> = serde_json::from_str(&cache_content)?;
    
    Ok(articles)
}

fn search_articles<'a>(articles: &'a [Article], query: &str) -> Vec<&'a Article> {
    let query_lower = query.to_lowercase();
    
    articles
        .iter()
        .filter(|article| {
            !article.published && (
                article.title.to_lowercase().contains(&query_lower) ||
                article.body_markdown.as_ref().is_some_and(|body| {
                    body.to_lowercase().contains(&query_lower)
                }) ||
                article.tags.as_ref().unwrap_or(&vec![]).iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            )
        })
        .collect()
}

fn get_draft_articles(articles: &[Article]) -> Vec<&Article> {
    articles
        .iter()
        .filter(|article| !article.published)
        .collect()
}

fn display_articles(articles: &[&Article]) {
    if articles.is_empty() {
        println!("{}", "No draft articles found.".yellow());
        return;
    }

    println!("{} draft article(s) found:\n", articles.len().to_string().green().bold());

    for (i, article) in articles.iter().enumerate() {
        println!("{}. {}", i + 1, article.title.cyan().bold());
        let edit_url = format!("https://dev.to/{}/{}/edit", article.user.username, article.slug);
        println!("{}", edit_url.blue().underline());
        println!();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set API key
    if let Some(api_key) = cli.set_api_key {
        let config = Config { api_key };
        save_config(&config).context("Failed to save API key")?;
        println!("{}", "API key saved successfully!".green());
        return Ok(());
    }

    // Load config
    let config = load_config().context("Failed to load configuration")?;

    // Get articles (from cache or API)
    let prev_cache_count = load_articles_cache().map(|a| a.len()).unwrap_or(0);
    if cli.refresh && prev_cache_count > 0 {
        let est_pages = (prev_cache_count as f64 / 1000.0).ceil() as u64;
        let est_time = est_pages;
        println!(
            "Current cache: {} articles. Estimated time to refresh: about {} seconds ({} pages).",
            prev_cache_count, est_time, est_pages
        );
    }
    let articles = if cli.refresh || load_articles_cache().unwrap_or_default().is_empty() {
        println!("{}", "Fetching articles from dev.to...".blue());
        let client = DevToClient::new(config.api_key);
        let articles = client.get_my_articles().await?;
        save_articles_cache(&articles).context("Failed to save articles cache")?;
        println!("{}", "Articles cached successfully!".green());
        articles
    } else {
        load_articles_cache().context("Failed to load articles cache")?
    };

    // Filter and display articles
    if cli.all {
        let drafts = get_draft_articles(&articles);
        display_articles(&drafts);
    } else if let Some(query) = cli.query {
        let filtered_articles = search_articles(&articles, &query);
        display_articles(&filtered_articles);
    } else {
        println!("{}", "Usage:".yellow().bold());
        println!("  dtdrafts -q <query>    Search draft articles");
        println!("  dtdrafts --all         Show all draft articles");
        println!("  dtdrafts --refresh     Refresh article cache");
        println!("  dtdrafts --set-api-key <key>  Set dev.to API key");
        println!();
        println!("{}", "Examples:".yellow().bold());
        println!("  dtdrafts -q aws");
        println!("  dtdrafts -q rust");
        println!("  dtdrafts --all");
    }

    Ok(())
}

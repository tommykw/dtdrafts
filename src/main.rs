use clap::Parser;
use colored::*;
use anyhow::{Result, Context};
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
pub struct Article {
    pub id: u64,
    pub title: String,
    pub description: Option<String>,
    pub body_markdown: Option<String>,
    pub url: String,
    pub canonical_url: Option<String>,
    pub url_with_preview: Option<String>,
    pub published: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub tags: Option<Vec<String>>,
    pub slug: String,
    pub user: ArticleUser,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ArticleUser {
    pub username: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub api_key: String,
}

pub struct DevToClient {
    client: reqwest::Client,
    pub api_key: String,
}

impl DevToClient {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::new();
        Self { client, api_key }
    }

    pub async fn get_my_articles(&self) -> Result<Vec<Article>> {
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
            tokio::time::sleep(std::time::Duration::from_secs(1)).await; // rate limit mitigation
        }

        println!("Done! Total {} articles fetched.", all_articles.len());
        Ok(all_articles)
    }
}

pub fn get_config_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let config_dir = home_dir.join(".dtdrafts");
    Ok(config_dir)
}

pub fn get_config_file() -> Result<PathBuf> {
    let mut config_file = get_config_dir()?;
    config_file.push("config.json");
    Ok(config_file)
}

pub fn get_cache_file() -> Result<PathBuf> {
    let mut cache_file = get_config_dir()?;
    cache_file.push("articles_cache.json");
    Ok(cache_file)
}

pub fn save_config(config: &Config) -> Result<()> {
    let config_dir = get_config_dir()?;
    fs::create_dir_all(&config_dir)?;
    let config_file = get_config_file()?;
    let config_json = serde_json::to_string_pretty(config)?;
    fs::write(config_file, config_json)?;
    Ok(())
}

pub fn load_config() -> Result<Config> {
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

pub fn save_articles_cache(articles: &[Article]) -> Result<()> {
    let config_dir = get_config_dir()?;
    fs::create_dir_all(&config_dir)?;
    let cache_file = get_cache_file()?;
    let cache_json = serde_json::to_string_pretty(articles)?;
    fs::write(cache_file, cache_json)?;
    Ok(())
}

pub fn load_articles_cache() -> Result<Vec<Article>> {
    let cache_file = get_cache_file()?;
    if !cache_file.exists() {
        return Ok(Vec::new());
    }
    let cache_content = fs::read_to_string(cache_file)?;
    let articles: Vec<Article> = serde_json::from_str(&cache_content)?;
    Ok(articles)
}

pub fn search_articles<'a>(articles: &'a [Article], query: &str) -> Vec<&'a Article> {
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

pub fn get_draft_articles(articles: &[Article]) -> Vec<&Article> {
    articles
        .iter()
        .filter(|article| !article.published)
        .collect()
}

pub fn display_articles(articles: &[&Article]) {
    if articles.is_empty() {
        println!("{}", "No draft articles found.".yellow());
        return;
    }
    println!("{} draft article(s) found:\n", articles.len().to_string().green().bold());
    for (i, article) in articles.iter().enumerate() {
        println!("{}. {}", i + 1, article.title.cyan().bold());
        let edit_url = format!("@https://dev.to/{}/{}/edit", article.user.username, article.slug);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_articles() -> Vec<Article> {
        vec![
            Article {
                id: 1,
                title: "Rust Tips".to_string(),
                description: Some("Learn Rust".to_string()),
                body_markdown: Some("Rust is great for CLI tools.".to_string()),
                url: "https://dev.to/user/rust-tips".to_string(),
                canonical_url: None,
                url_with_preview: None,
                published: false,
                created_at: None,
                updated_at: None,
                tags: Some(vec!["rust".to_string(), "cli".to_string()]),
                slug: "rust-tips".to_string(),
                user: ArticleUser { username: "user".to_string() },
            },
            Article {
                id: 2,
                title: "Kotlin Guide".to_string(),
                description: Some("Kotlin basics".to_string()),
                body_markdown: Some("Kotlin is a modern language.".to_string()),
                url: "https://dev.to/user/kotlin-guide".to_string(),
                canonical_url: None,
                url_with_preview: None,
                published: true,
                created_at: None,
                updated_at: None,
                tags: Some(vec!["kotlin".to_string(), "android".to_string()]),
                slug: "kotlin-guide".to_string(),
                user: ArticleUser { username: "user".to_string() },
            },
            Article {
                id: 3,
                title: "CLI Tricks".to_string(),
                description: Some("Tips for CLI".to_string()),
                body_markdown: Some("Use Rust or Python for CLI.".to_string()),
                url: "https://dev.to/user/cli-tricks".to_string(),
                canonical_url: None,
                url_with_preview: None,
                published: false,
                created_at: None,
                updated_at: None,
                tags: Some(vec!["cli".to_string(), "tools".to_string()]),
                slug: "cli-tricks".to_string(),
                user: ArticleUser { username: "user".to_string() },
            },
        ]
    }

    #[test]
    fn test_search_by_title() {
        let articles = sample_articles();
        let found = search_articles(&articles, "rust");
        assert_eq!(found.len(), 2);
        let titles: Vec<_> = found.iter().map(|a| a.title.as_str()).collect();
        assert!(titles.contains(&"Rust Tips"));
        assert!(titles.contains(&"CLI Tricks"));
    }

    #[test]
    fn test_search_by_body_markdown() {
        let articles = sample_articles();
        let found = search_articles(&articles, "modern language");
        // Only published article has this, so should not be found
        assert_eq!(found.len(), 0);
        let found2 = search_articles(&articles, "python");
        assert_eq!(found2.len(), 1);
        assert_eq!(found2[0].title, "CLI Tricks");
    }

    #[test]
    fn test_search_by_tag() {
        let articles = sample_articles();
        let found = search_articles(&articles, "cli");
        // Two unpublished articles have 'cli' tag
        assert_eq!(found.len(), 2);
        let titles: Vec<_> = found.iter().map(|a| a.title.as_str()).collect();
        assert!(titles.contains(&"Rust Tips"));
        assert!(titles.contains(&"CLI Tricks"));
    }

    #[test]
    fn test_get_draft_articles() {
        let articles = sample_articles();
        let drafts = get_draft_articles(&articles);
        assert_eq!(drafts.len(), 2);
        let titles: Vec<_> = drafts.iter().map(|a| a.title.as_str()).collect();
        assert!(titles.contains(&"Rust Tips"));
        assert!(titles.contains(&"CLI Tricks"));
    }
}

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
            let url = format!("{base_url}/articles/me/unpublished?page={page}&per_page={per_page}");
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
    use colored::*;
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
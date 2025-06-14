use clap::Parser;
use colored::*;
use anyhow::{Result, Context};
use dtdrafts::*;

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


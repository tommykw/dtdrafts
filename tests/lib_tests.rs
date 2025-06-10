use dtdrafts::*;

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
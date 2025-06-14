# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

dtdrafts is a Rust CLI tool for searching and managing unpublished (draft) articles from dev.to. The application fetches articles via the dev.to API, caches them locally, and provides search functionality.

## Architecture

- **src/main.rs**: CLI interface using clap, handles command parsing and orchestrates the application flow
- **src/lib.rs**: Core library containing all business logic, data structures, and API client
- **tests/lib_tests.rs**: Integration tests for the core functionality

Key components:
- `DevToClient`: HTTP client for dev.to API interactions with rate limiting
- `Article`/`ArticleUser`/`Config`: Data structures matching dev.to API responses
- Search functions: Filter articles by title, body content, and tags (only unpublished articles)
- Cache management: Local JSON storage in `~/.dtdrafts/` directory

## Common Commands

### Development
```bash
# Build the project (debug)
cargo build

# Build optimized release version
cargo build --release

# Run tests
cargo test

# Run a specific test
cargo test test_search_by_title

# Run the application in dev mode
cargo run -- --help

# Run the built binary
./target/release/dtdrafts --help
```

### Testing the CLI
```bash
# Set API key for testing
cargo run -- --set-api-key YOUR_DEV_TO_API_KEY

# Search drafts
cargo run -- -q rust
cargo run -- --all

# Force refresh cache
cargo run -- --refresh --all
```

## Configuration

The application stores configuration and cache in `~/.dtdrafts/`:
- `config.json`: Contains the dev.to API key
- `articles_cache.json`: Cached article data from dev.to API

API key can be obtained from dev.to Settings > Extensions.

## Key Implementation Details

- All functions in lib.rs are marked `pub` for external access but many could be made private if not used by main.rs or tests
- The dev.to API is paginated (1000 articles per page) with rate limiting (1 second delay between requests)  
- Search functionality only operates on unpublished articles (`published: false`)
- Article fetching includes progress reporting during multi-page API calls
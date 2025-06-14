# dtdrafts

A Rust command-line tool to search and list your unpublished (draft) articles on [dev.to](https://dev.to/).

## Features
- Search and list your unpublished (draft) articles from dev.to locally
- Securely store your dev.to API key
- Article caching for faster access
- Fast full-text search

## Installation

1. Clone this repository or download the source code
2. Install dependencies (see below)
3. Build the project

```sh
cargo build --release
```

## Dependencies
- clap
- anyhow
- colored
- serde / serde_json
- reqwest
- dirs
- tokio

## Usage

### 1. Set your dev.to API key
First, save your API key:

```sh
./target/release/dtdrafts --set-api-key YOUR_API_KEY
```

### 2. Search your draft articles

#### Search by keyword
```sh
./target/release/dtdrafts -q rust
```

#### Show all draft articles
```sh
./target/release/dtdrafts --all
```

#### Force refresh the article cache
```sh
./target/release/dtdrafts --refresh -q aws
```

### 3. Show help
```sh
./target/release/dtdrafts --help
```

## Config & Cache File Locations
- Config file: `~/.dtdrafts/config.json`
- Cache file: `~/.dtdrafts/articles_cache.json`

### About `~/.dtdrafts/config.json`
This file stores your dev.to API key. You can set it using the CLI:

```sh
./target/release/dtdrafts --set-api-key YOUR_API_KEY
```

Or, you can manually create/edit the file at `~/.dtdrafts/config.json` with the following content:

```json
{
  "api_key": "YOUR_API_KEY"
}
```

If you ever want to remove your credentials, simply delete this file:

```sh
rm ~/.dtdrafts/config.json
```

## Notes
- You can get your dev.to API key from [dev.to Settings > Extensions](https://dev.to/settings/extensions).
- This tool is intended for personal use.

## License
MIT 
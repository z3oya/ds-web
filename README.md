# ds-web

A minimal CLI for the [DeepSeek Search & Fetch API](https://api-docs.deepseek.com): keyword web search, single-page fetch to Markdown, and balance lookup.

## Build

```
cargo build --release
```

## Setup

Set your DeepSeek API key:

```
export DEEPSEEK_API_KEY=sk-...
```

Note: each call bills your DeepSeek account balance.

## Usage

```
# Web search
ds-web search "rust async runtime" --count 5

# Fetch a page as Markdown
ds-web fetch "https://example.com"

# Check balance
ds-web status

# Raw JSON output (any subcommand)
ds-web search "query" --json
```

Queries support operators such as `site:`, `-site:`, `"exact phrase"`, `lang:zh,en`, `after:2025-01-01`. See [skills/ds-web-skill/SKILL.md](skills/ds-web-skill/SKILL.md) for the full reference.

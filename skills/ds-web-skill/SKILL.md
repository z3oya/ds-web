---
name: ds-web-skill
description: |
  Web search and page fetch through the ds-web CLI (DeepSeek Search & Fetch API).
  Covers keyword search with query operators (site:, -site:, -term, "exact phrase",
  lang:, after:, before:), single-page fetch to Markdown, result fields
  (rerank_score, authority, date, summary), and account balance via the status
  subcommand.
  Use when: a task needs web search, current web information, page content from a
  known URL, or a DeepSeek balance lookup.
---

# ds-web

`ds-web` calls the DeepSeek Search & Fetch API at `https://api.deepseek.com`.
Authentication reads the `DEEPSEEK_API_KEY` environment variable. Each call
bills the DeepSeek account balance; the status subcommand reports the balance.

## search

```
ds-web search "QUERY" [--count N] [--content] [--ori-question S] [--localization IP] [--json]
```

- `query`: 1-128 characters; query operators pass through verbatim
- `--count`: result count, 1-50, default 10
- `--content`: include page content in each result (`need_content`)
- `--ori-question`: the user's original question, up to 128 characters, improves query understanding
- `--localization`: IP address used to localize results
- `--json`: print the raw JSON response

Query operators:

| Operator | Example | Effect |
|---|---|---|
| `site:` | `DeepSeek site:github.com` | restrict to a site; join several with `OR` |
| `-site:` | `DeepSeek -site:example.com` | exclude a site |
| `-term` | `DeepSeek -advertisement` | exclude pages containing the term |
| `-"..."` | `DeepSeek -"release notes"` | exclude an exact phrase |
| `"..."` | `"exact phrase"` | match an exact phrase |
| `lang:` | `DeepSeek lang:zh,en` | restrict to language codes |
| `after:` | `DeepSeek after:2025-01-01` | pages published after a date |
| `before:` | `DeepSeek before:2026-06-01` | pages published before a date |

Each result carries: `rerank_score` (0-3; the API docs treat 1.5 and above as
high-relevance), `title`, `url`, `snippet` (100-200 characters, display-oriented),
`summary` (600-800 characters, model-oriented), `content` (with `--content`),
`site_name`, `site_icon`, `date` (RFC 3339), and `authority` (0-4; the API docs
treat 3 and above as authoritative, -1 unknown).

## fetch

```
ds-web fetch "URL" [--json]
```

Returns the page title and content as Markdown. The URL accepts HTTP and HTTPS
with a host, up to 8192 characters; URLs with userinfo (`user:pass@host`)
produce an API error.

## status

```
ds-web status [--json]
```

Prints `is_available` and per-currency balances in the form
`total = granted + topped up`.

## Selection

- `search` finds pages for a query; `fetch` reads one known URL.
- Combine them: search first, then fetch the result URLs that matter.
- `--json` yields the full response for programmatic processing.

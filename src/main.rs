use std::time::Duration;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use serde_json::{Value, json};

const BASE: &str = "https://api.deepseek.com";
const TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Parser)]
#[command(name = "ds-web", version, about = "DeepSeek Search & Fetch CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    Search {
        query: String,
        #[arg(long)]
        count: Option<u32>,
        #[arg(long)]
        content: bool,
        #[arg(long)]
        ori_question: Option<String>,
        #[arg(long)]
        localization: Option<String>,
    },
    Fetch {
        url: String,
    },
    Status,
}

fn auth() -> Result<String> {
    let key = std::env::var("DEEPSEEK_API_KEY")
        .context("DEEPSEEK_API_KEY environment variable not set")?;
    Ok(format!("Bearer {key}"))
}

fn send(req: ureq::Request, body: Option<String>) -> Result<Value> {
    let attempted = match body {
        Some(b) => req.timeout(TIMEOUT).send_string(&b),
        None => req.timeout(TIMEOUT).call(),
    };
    match attempted {
        Ok(resp) => Ok(resp.into_json()?),
        Err(ureq::Error::Status(code, resp)) => {
            let text = resp.into_string().unwrap_or_default();
            let head: String = text.chars().take(300).collect();
            bail!("HTTP {code}: {head}");
        }
        Err(e) => Err(e.into()),
    }
}

fn post_json(path: &str, payload: &Value) -> Result<Value> {
    let auth = auth()?;
    send(
        ureq::post(&format!("{BASE}{path}"))
            .set("Authorization", &auth)
            .set("Content-Type", "application/json"),
        Some(payload.to_string()),
    )
    .with_context(|| format!("request to {path} failed"))
}

fn get_json(path: &str) -> Result<Value> {
    let auth = auth()?;
    send(
        ureq::get(&format!("{BASE}{path}")).set("Authorization", &auth),
        None,
    )
    .with_context(|| format!("request to {path} failed"))
}

fn print_json(v: &Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(v)?);
    Ok(())
}

fn cmd_search(
    query: &str,
    count: Option<u32>,
    content: bool,
    ori_question: Option<&str>,
    localization: Option<&str>,
    as_json: bool,
) -> Result<()> {
    let mut payload = json!({ "query": query });
    if let Some(c) = count {
        payload["count"] = json!(c);
    }
    if content {
        payload["need_content"] = json!(true);
    }
    if let Some(q) = ori_question {
        payload["ori_question"] = json!(q);
    }
    if let Some(ip) = localization {
        payload["localization"] = json!(ip);
    }
    let data = post_json("/v1/tools/search", &payload)?;
    if as_json {
        return print_json(&data);
    }
    let Some(results) = data["results"].as_array() else {
        return Ok(());
    };
    if results.is_empty() {
        println!("(no results)");
        return Ok(());
    }
    for (i, it) in results.iter().enumerate() {
        println!("[{}] {}", i + 1, it["title"].as_str().unwrap_or(""));
        println!("    {}", it["url"].as_str().unwrap_or(""));
        let mut extra: Vec<String> = Vec::new();
        if let Some(s) = it["rerank_score"].as_f64() {
            extra.push(format!("score={s}"));
        }
        if let Some(a) = it["authority"].as_i64() {
            extra.push(format!("authority={a}"));
        }
        if let Some(d) = it["date"].as_str() {
            extra.push(format!("date={}", d.get(..10).unwrap_or(d)));
        }
        if let Some(s) = it["site_name"].as_str() {
            extra.push(format!("site={s}"));
        }
        if !extra.is_empty() {
            println!("    {}", extra.join("  "));
        }
        if let Some(s) = it["summary"].as_str() {
            println!("    {s}");
            println!();
        }
    }
    Ok(())
}

fn cmd_fetch(url: &str, as_json: bool) -> Result<()> {
    let data = post_json("/v1/tools/fetch", &json!({ "url": url }))?;
    if as_json {
        return print_json(&data);
    }
    if let Some(t) = data["title"].as_str().filter(|t| !t.is_empty()) {
        println!("# {t}");
        println!();
    }
    match data["content"].as_str() {
        Some(c) if !c.is_empty() => println!("{c}"),
        _ => print_json(&data)?,
    }
    Ok(())
}

fn cmd_status(as_json: bool) -> Result<()> {
    let data = get_json("/user/balance")?;
    if as_json {
        return print_json(&data);
    }
    println!(
        "available: {}",
        data["is_available"].as_bool().unwrap_or(false)
    );
    if let Some(infos) = data["balance_infos"].as_array() {
        for info in infos {
            let cur = info["currency"].as_str().unwrap_or("?");
            println!(
                "balance ({cur}): total {} = granted {} + topped up {}",
                info["total_balance"].as_str().unwrap_or("?"),
                info["granted_balance"].as_str().unwrap_or("?"),
                info["topped_up_balance"].as_str().unwrap_or("?"),
            );
        }
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Search {
            query,
            count,
            content,
            ori_question,
            localization,
        } => cmd_search(
            &query,
            count,
            content,
            ori_question.as_deref(),
            localization.as_deref(),
            cli.json,
        ),
        Commands::Fetch { url } => cmd_fetch(&url, cli.json),
        Commands::Status => cmd_status(cli.json),
    };
    if let Err(e) = result {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

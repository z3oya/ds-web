use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde_json::{json, Value};

const BASE: &str = "https://api.deepseek.com";

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
    },
    Fetch {
        url: String,
    },
    Status,
}

fn api_key() -> Result<String> {
    std::env::var("DEEPSEEK_API_KEY").context("缺少 DEEPSEEK_API_KEY 环境变量")
}

fn take_body(resp: ureq::Response) -> Result<Value> {
    Ok(resp.into_json()?)
}

fn fail_status(code: u16, resp: ureq::Response) -> anyhow::Error {
    let body = resp.into_string().unwrap_or_default();
    let head: String = body.chars().take(300).collect();
    anyhow::anyhow!("HTTP {code}: {head}")
}

fn post_json(path: &str, payload: &Value) -> Result<Value> {
    match ureq::post(&format!("{BASE}{path}"))
        .set("Authorization", &format!("Bearer {}", api_key()?))
        .set("Content-Type", "application/json")
        .timeout(Duration::from_secs(120))
        .send_string(&payload.to_string())
    {
        Ok(resp) => take_body(resp),
        Err(ureq::Error::Status(code, resp)) => Err(fail_status(code, resp)),
        Err(e) => Err(e.into()),
    }
    .context(format!("请求 {path} 失败"))
}

fn get_json(path: &str) -> Result<Value> {
    match ureq::get(&format!("{BASE}{path}"))
        .set("Authorization", &format!("Bearer {}", api_key()?))
        .timeout(Duration::from_secs(60))
        .call()
    {
        Ok(resp) => take_body(resp),
        Err(ureq::Error::Status(code, resp)) => Err(fail_status(code, resp)),
        Err(e) => Err(e.into()),
    }
    .context(format!("请求 {path} 失败"))
}

fn print_json(v: &Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(v)?);
    Ok(())
}

fn cmd_search(query: &str, count: Option<u32>, content: bool, as_json: bool) -> Result<()> {
    let mut payload = json!({ "query": query });
    if let Some(c) = count {
        payload["count"] = json!(c);
    }
    if content {
        payload["need_content"] = json!(true);
    }
    let data = post_json("/v1/tools/search", &payload)?;
    if as_json {
        return print_json(&data);
    }
    let results = data["results"].as_array().cloned().unwrap_or_default();
    if results.is_empty() {
        println!("(无结果)");
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
            extra.push(format!("date={}", &d[..d.len().min(10)]));
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
    if let Some(t) = data["title"].as_str() {
        if !t.is_empty() {
            println!("# {t}");
            println!();
        }
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
    println!("可用: {}", data["is_available"].as_bool().unwrap_or(false));
    if let Some(infos) = data["balance_infos"].as_array() {
        for info in infos {
            let cur = info["currency"].as_str().unwrap_or("?");
            println!(
                "余额 ({cur}): 总 {} = 赠送 {} + 充值 {}",
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
        } => cmd_search(&query, count, content, cli.json),
        Commands::Fetch { url } => cmd_fetch(&url, cli.json),
        Commands::Status => cmd_status(cli.json),
    };
    if let Err(e) = result {
        eprintln!("错误: {e:#}");
        std::process::exit(1);
    }
}

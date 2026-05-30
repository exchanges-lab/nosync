use anyhow::{Context, Result};
use dotenvy::dotenv;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use std::env;

fn format_db_id(id: &str) -> String {
    let cleaned = id.replace("-", "");
    if cleaned.len() == 32 {
        format!(
            "{}-{}-{}-{}-{}",
            &cleaned[0..8],
            &cleaned[8..12],
            &cleaned[12..16],
            &cleaned[16..20],
            &cleaned[20..32]
        )
    } else {
        id.to_string()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();

    let token = env::var("NOTION_API_KEY").context("Missing NOTION_API_KEY")?;
    let db_id_raw = env::var("NOTION_DATABASE_ID").context("Missing NOTION_DATABASE_ID")?;
    let db_id = format_db_id(&db_id_raw);

    // Setup headers
    let mut headers = HeaderMap::new();
    headers.insert("Notion-Version", HeaderValue::from_static("2022-06-28"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token))?,
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    // 1. Try GET /v1/databases/{db_id}
    let get_url = format!("https://api.notion.com/v1/databases/{}", db_id);
    println!("Sending GET request to: {} ...", get_url);
    let get_res = client.get(&get_url).send().await?;
    let get_status = get_res.status();
    let get_body = get_res.text().await?;
    println!("GET Status: {}", get_status);
    println!("GET Body:\n{}", get_body);

    println!("\n--------------------------------------------------\n");

    // 2. Try POST /v1/databases/{db_id}/query
    let post_url = format!("https://api.notion.com/v1/databases/{}/query", db_id);
    println!("Sending POST request to: {} ...", post_url);
    let post_res = client
        .post(&post_url)
        .header("Content-Type", "application/json")
        .body("{}")
        .send()
        .await?;
    let post_status = post_res.status();
    let post_body = post_res.text().await?;
    println!("POST Status: {}", post_status);
    println!("POST Body:\n{}", post_body);

    Ok(())
}

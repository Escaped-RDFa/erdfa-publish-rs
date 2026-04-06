//! mediawiki-fetch — Generic MediaWiki API fetcher with ERDFA/DASL envelope output.
//! Flavors: imslp, fandom, archiveteam, or any custom MediaWiki endpoint.
//!
//! Uses the same Seal pattern as opendatasync/src/erdfa.rs.

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use erdfa_publish::{Shard, ShardSet, Component};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

#[derive(ValueEnum, Clone, Debug)]
enum Flavor {
    Imslp,
    Fandom,
    Archiveteam,
    Custom,
}

#[derive(Parser)]
#[command(name = "mediawiki-fetch", about = "Fetch MediaWiki metadata → ERDFA shards")]
struct Cli {
    /// MediaWiki flavor
    #[arg(value_enum)]
    flavor: Flavor,

    /// Page titles or category (comma-separated)
    #[arg(short, long)]
    titles: Option<String>,

    /// Category to enumerate (e.g. "Pozzoli, Ettore")
    #[arg(short, long)]
    category: Option<String>,

    /// Fandom wiki subdomain (e.g. "buckaroobanzai")
    #[arg(long)]
    wiki: Option<String>,

    /// Custom API URL (for --flavor custom)
    #[arg(long)]
    api_url: Option<String>,

    /// Output directory for shards
    #[arg(short, long, default_value = "shards/mediawiki")]
    output: PathBuf,
}

fn api_base(cli: &Cli) -> String {
    match cli.flavor {
        Flavor::Imslp => "https://imslp.org/api.php".into(),
        Flavor::Fandom => format!(
            "https://{}.fandom.com/api.php",
            cli.wiki.as_deref().unwrap_or("buckaroobanzai")
        ),
        Flavor::Archiveteam => "https://wiki.archiveteam.org/api.php".into(),
        Flavor::Custom => cli.api_url.clone().unwrap_or_else(|| "https://en.wikipedia.org/w/api.php".into()),
    }
}

/// Seal: SHA-256 witness + orbifold coords (same as opendatasync erdfa.rs)
fn seal(key: &str, url: &str, data: &[u8]) -> serde_json::Value {
    let h = Sha256::digest(data);
    let v = u64::from_le_bytes(h[0..8].try_into().unwrap());
    serde_json::json!({
        "key": key,
        "url": url,
        "witness": hex::encode(h),
        "dasl": format!("0xda51{:012x}", v & 0xffffffffffff),
        "orbifold": [v % 71, v % 59, v % 47],
        "size": data.len(),
    })
}

/// Query category members via MediaWiki API
fn fetch_category(base: &str, cat: &str) -> Result<Vec<String>> {
    let url = format!(
        "{}?action=query&list=categorymembers&cmtitle=Category:{}&cmlimit=500&format=json",
        base,
        urlenc(cat)
    );
    let resp: serde_json::Value = ureq::get(&url).call()?.into_json()?;
    let members = resp["query"]["categorymembers"]
        .as_array()
        .map(|a| a.iter().filter_map(|m| m["title"].as_str().map(String::from)).collect())
        .unwrap_or_default();
    Ok(members)
}

/// Query page content/metadata via MediaWiki API
fn fetch_pages(base: &str, titles: &[String]) -> Result<Vec<(String, serde_json::Value)>> {
    let mut results = Vec::new();
    for chunk in titles.chunks(20) {
        let joined = chunk.iter().map(|t| urlenc(t)).collect::<Vec<_>>().join("|");
        let url = format!(
            "{}?action=query&titles={}&prop=revisions|categories|info&rvprop=content|timestamp&rvslots=main&format=json",
            base, joined
        );
        let resp: serde_json::Value = ureq::get(&url).call()?.into_json()?;
        if let Some(pages) = resp["query"]["pages"].as_object() {
            for (_id, page) in pages {
                let title = page["title"].as_str().unwrap_or("unknown").to_string();
                results.push((title, page.clone()));
            }
        }
    }
    Ok(results)
}

fn urlenc(s: &str) -> String {
    s.replace(' ', "_").replace('&', "%26").replace('=', "%3D")
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let base = api_base(&cli);
    eprintln!("[mediawiki-fetch] flavor={:?} api={}", cli.flavor, base);

    // Resolve titles
    let titles: Vec<String> = if let Some(cat) = &cli.category {
        eprintln!("[mediawiki-fetch] enumerating Category:{}", cat);
        fetch_category(&base, cat).context("category fetch")?
    } else if let Some(t) = &cli.titles {
        t.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        anyhow::bail!("provide --titles or --category");
    };

    eprintln!("[mediawiki-fetch] {} pages to fetch", titles.len());
    let pages = fetch_pages(&base, &titles).context("page fetch")?;

    // Output
    std::fs::create_dir_all(&cli.output)?;
    let mut shard_set = ShardSet::new(format!("mediawiki-{:?}", cli.flavor).to_lowercase());
    let mut shards = Vec::new();

    for (title, page) in &pages {
        let json_bytes = serde_json::to_vec_pretty(page)?;
        let s = seal(&title, &base, &json_bytes);

        // Write raw JSON
        let safe_name: String = title.chars().map(|c| if c.is_alphanumeric() || c == '.' || c == '-' { c } else { '_' }).collect();
        let raw_path = cli.output.join(format!("{}.json", safe_name));
        std::fs::write(&raw_path, &json_bytes)?;

        // Write seal
        let seal_path = cli.output.join(format!("{}.seal.json", safe_name));
        std::fs::write(&seal_path, serde_json::to_string_pretty(&s)?)?;

        // ERDFA shard
        let shard = Shard::new(
            &safe_name,
            Component::KeyValue {
                pairs: vec![
                    ("title".into(), title.clone()),
                    ("source".into(), base.clone()),
                    ("witness".into(), s["witness"].as_str().unwrap_or("").into()),
                    ("dasl".into(), s["dasl"].as_str().unwrap_or("").into()),
                ],
            },
        ).with_tags(vec![
            format!("{:?}", cli.flavor).to_lowercase(),
            "mediawiki".into(),
            title.clone(),
        ]);
        shard_set.add(&shard);
        shards.push(shard);

        eprintln!("  ✓ {} orb=({},{},{}) size={}",
            title, s["orbifold"][0], s["orbifold"][1], s["orbifold"][2], s["size"]);
    }

    // Write manifest
    let manifest_path = cli.output.join("manifest.json");
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&shard_set)?)?;

    // Write CBOR tar
    let tar_path = cli.output.join("shards.tar");
    let f = std::fs::File::create(&tar_path)?;
    shard_set.to_tar(&shards, f)?;

    eprintln!("[mediawiki-fetch] {} shards → {}", shards.len(), cli.output.display());
    Ok(())
}

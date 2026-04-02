//! da51-git-rehash — Rehash an entire git repo with Monster Hash DA51 addresses.
//!
//! Uses libgit2 to walk every object. Pure Rust, no bash.
//!
//! Usage: da51-git-rehash <repo_path> [--output <da51_index.jsonl>]

use clap::Parser;
use erdfa_publish::da51_macros::*;
use erdfa_publish::da51_hash;
use git2::{ObjectType, Repository};
use serde_json::json;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "da51-git-rehash", about = "Rehash a git repo with Monster Hash DA51 addresses")]
struct Cli {
    /// Path to git repository
    repo: PathBuf,
    /// Output JSONL file (default: <repo>/.git/da51_index.jsonl)
    #[arg(long)]
    output: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    let repo = Repository::open(&cli.repo).expect("open repo");
    let output_path = cli.output.unwrap_or_else(|| cli.repo.join(".git/da51_index.jsonl"));
    let file = File::create(&output_path).expect("create output");
    let mut out = BufWriter::new(file);

    let hasher = da51_hash!(8, 4, 3);
    let cell = GriessCell::new(8, 4, 3);

    let odb = repo.odb().expect("odb");
    let mut total = 0u64;
    let mut blobs = 0u64;
    let mut trees = 0u64;
    let mut commits = 0u64;
    let mut tags = 0u64;

    odb.foreach(|oid| {
        let obj = match odb.read(*oid) {
            Ok(o) => o,
            Err(_) => return true,
        };

        let data = obj.data();
        let hash = hasher(data);
        let folded = cell.fold(&hash);
        let dasl = format!("0xda51{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            folded[0], folded[1], folded[2], folded[3], folded[4], folded[5]);

        let type_str = match obj.kind() {
            ObjectType::Blob => { blobs += 1; "blob" }
            ObjectType::Tree => { trees += 1; "tree" }
            ObjectType::Commit => { commits += 1; "commit" }
            ObjectType::Tag => { tags += 1; "tag" }
            _ => "unknown",
        };

        let line = json!({
            "sha": oid.to_string(),
            "da51": dasl,
            "orb": [folded[0], folded[1], folded[2]],
            "bott": folded[2] % 8,
            "type": type_str,
            "size": data.len(),
        });

        writeln!(out, "{}", line).ok();
        total += 1;

        if total % 1000 == 0 {
            eprint!("\r[da51-git-rehash] {} objects...", total);
        }

        true // continue
    }).expect("foreach");

    out.flush().expect("flush");

    eprintln!("\r[da51-git-rehash] {} objects ({} blobs, {} trees, {} commits, {} tags)",
        total, blobs, trees, commits, tags);
    eprintln!("[da51-git-rehash] → {}", output_path.display());
}

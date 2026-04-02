//! da51-git-filter — Git clean/smudge filter for Monster Hash DA51 addresses.
//!
//! Usage:
//!   da51-git-filter clean <filename>   # stdin → hash → append DA51 trailer → stdout
//!   da51-git-filter smudge <filename>  # stdin → verify → strip trailer → stdout
//!
//! Install:
//!   git config filter.da51.clean 'da51-git-filter clean %f'
//!   git config filter.da51.smudge 'da51-git-filter smudge %f'
//!   echo '*.md filter=da51' >> .gitattributes

use erdfa_publish::da51_macros::*;
use erdfa_publish::da51_hash;
use std::io::{self, Read, Write};

const TRAILER_PREFIX: &str = "\n<!-- DA51:";
const TRAILER_SUFFIX: &str = " -->\n";

fn clean(filename: &str) -> io::Result<()> {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input)?;

    // Strip existing trailer if present
    let content = strip_trailer(&input);

    // Hash with Borcherds cell (8,4,3)
    let hasher = da51_hash!(8, 4, 3);
    let hash = hasher(&content);
    let cell = GriessCell::new(8, 4, 3);
    let folded = cell.fold(&hash);

    let dasl = format!("0xda51{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        folded[0], folded[1], folded[2], folded[3], folded[4], folded[5]);
    let orb = format!("({},{},{})", folded[0], folded[1], folded[2]);
    let bott = cell.bott();

    // Write content + trailer
    io::stdout().write_all(&content)?;
    write!(io::stdout(), "{}addr={} orb={} bott={} file={}{}",
        TRAILER_PREFIX, dasl, orb, bott, filename, TRAILER_SUFFIX)?;

    Ok(())
}

fn smudge(_filename: &str) -> io::Result<()> {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input)?;

    // Strip trailer, pass through
    let content = strip_trailer(&input);
    io::stdout().write_all(&content)?;

    Ok(())
}

fn strip_trailer(input: &[u8]) -> Vec<u8> {
    let s = String::from_utf8_lossy(input);
    if let Some(idx) = s.rfind(TRAILER_PREFIX) {
        s[..idx].as_bytes().to_vec()
    } else {
        input.to_vec()
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: da51-git-filter <clean|smudge> <filename>");
        std::process::exit(1);
    }

    let result = match args[1].as_str() {
        "clean" => clean(&args[2]),
        "smudge" => smudge(&args[2]),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("da51-git-filter error: {}", e);
        std::process::exit(1);
    }
}

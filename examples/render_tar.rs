use erdfa_publish::render::{render_text, render_html, decode_shard};
use std::io::Read;

fn main() {
    let tar_path = std::env::args().nth(1)
        .unwrap_or_else(|| "shards/example.tar".into());

    let data = std::fs::read(&tar_path).expect("cannot read tar");
    let mut pos = 0;
    let mut shards = Vec::new();

    // Parse tar entries (512-byte headers + data + padding)
    while pos + 512 <= data.len() {
        let header = &data[pos..pos + 512];
        // Empty block = end of tar
        if header.iter().all(|&b| b == 0) { break; }
        // Name
        let name_end = header[..100].iter().position(|&b| b == 0).unwrap_or(100);
        let name = std::str::from_utf8(&header[..name_end]).unwrap_or("?");
        // Size (octal)
        let size_str = std::str::from_utf8(&header[124..135]).unwrap_or("0").trim();
        let size = usize::from_str_radix(size_str.trim_end_matches('\0'), 8).unwrap_or(0);
        pos += 512;
        if pos + size > data.len() { break; }
        let entry_data = &data[pos..pos + size];
        // Pad to 512
        pos += (size + 511) / 512 * 512;

        if name.ends_with(".cbor") && name != "manifest.cbor" {
            if let Some(shard) = decode_shard(entry_data) {
                shards.push(shard);
            }
        }
    }

    println!("=== {} shards from {} ===\n", shards.len(), tar_path);

    // Text rendering
    println!("──── TEXT RENDERING ────\n");
    for shard in &shards {
        print!("{}", render_text(shard));
        println!();
    }

    // HTML rendering (write to file)
    let mut html = String::from("<!DOCTYPE html>\n<html><head><meta charset=\"utf-8\">\n<title>Shem HaMephorash — 72 Names</title>\n<style>\n  body { font-family: sans-serif; max-width: 900px; margin: 2em auto; }\n  article { border: 1px solid #ccc; padding: 1em; margin: 1em 0; border-radius: 4px; }\n  .map-entity { position: relative; display: inline-block; margin: 4px; padding: 4px 8px;\n    border-radius: 3px; font-size: 0.85em; }\n  .map-entity[data-kind=\"GOD\"] { background: #ffd; border: 1px solid #cc0; }\n  .map-entity[data-kind=\"PERSON\"] { background: #ddf; border: 1px solid #00c; }\n  .map-entity[data-kind=\"THING\"] { background: #dfd; border: 1px solid #0a0; }\n  .map-entity[data-kind=\"PLACE\"] { background: #fdd; border: 1px solid #c00; }\n  .map-entity[data-kind=\"VERB\"] { background: #edf; border: 1px solid #a0a; }\n  dt { font-weight: bold; display: inline; } dd { display: inline; margin: 0 1em 0 0; }\n  .meta { font-size: 0.8em; color: #666; display: block; }\n</style>\n</head><body>\n<h1>Shem HaMephorash — 72 Names (eRDFa CBOR Shards)</h1>\n");
    for shard in &shards {
        html.push_str(&render_html(shard));
    }
    html.push_str("</body></html>\n");

    let html_path = tar_path.replace(".tar", ".html");
    std::fs::write(&html_path, &html).expect("cannot write html");
    println!("──── HTML written to {} ({} bytes) ────", html_path, html.len());
}

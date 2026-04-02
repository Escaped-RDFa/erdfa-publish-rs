//! da51-ttl-erdfa — Export DA51 CBOR shards as RDF Turtle with full eRDFa annotations.
//!
//! Usage: da51-ttl-erdfa --dir <shard_dir> [--output <file.ttl>]
//!
//! Reads all .cbor files in a directory, decodes each Shard, and emits
//! Turtle triples with erdfa:SheafSection typing, DASL addresses,
//! orbifold coordinates, Bott periods, and CFT parent→child arrows.

use clap::Parser;
use erdfa_publish::render::decode_shard;
use erdfa_publish::{Component, Shard};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "da51-ttl-erdfa", about = "Export DA51 CBOR shards as RDF Turtle")]
struct Cli {
    /// Directory containing .cbor shard files
    #[arg(long)]
    dir: PathBuf,
    /// Output .ttl file (default: stdout)
    #[arg(long)]
    output: Option<PathBuf>,
}

/// Extract seal metadata from shard content (looks for erdfa-seal header fields)
fn extract_seal_field(shard: &Shard, field: &str) -> Option<String> {
    // Check KeyValue pairs for "content" key containing the seal header
    if let Component::KeyValue { pairs } = &shard.component {
        for (k, v) in pairs {
            if k == "content" || k == "bigrams" {
                for line in v.lines() {
                    if line.starts_with(field) {
                        return Some(line[field.len()..].trim().to_string());
                    }
                }
            }
        }
    }
    None
}

fn main() {
    let cli = Cli::parse();
    let mut out: Box<dyn Write> = match &cli.output {
        Some(p) => Box::new(fs::File::create(p).expect("create output")),
        None => Box::new(std::io::stdout()),
    };

    // Prefixes
    write!(out, "{}", r#"@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs:    <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix dc:      <http://purl.org/dc/elements/1.1/> .
@prefix erdfa:   <https://meta-introspector.github.io/erdfa/> .
@prefix dasl:    <https://meta-introspector.github.io/da51/> .
@prefix sheaf:   <https://meta-introspector.github.io/sheaf/> .
@prefix cft:     <https://meta-introspector.github.io/cft/> .
@prefix shard:   <https://meta-introspector.github.io/shard/> .

"#).unwrap();

    let bott_labels = ["ℤ", "ℤ/2", "ℤ/2", "0", "ℤ", "0", "0", "0"];

    let mut shard_count = 0u64;
    let mut arrow_count = 0u64;

    let mut entries: Vec<PathBuf> = fs::read_dir(&cli.dir)
        .expect("read dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map_or(false, |e| e == "cbor"))
        .collect();
    entries.sort();

    for path in &entries {
        let bytes = match fs::read(path) {
            Ok(b) => b,
            Err(_) => continue,
        };
        let shard = match decode_shard(&bytes) {
            Some(s) => s,
            None => continue,
        };

        let dasl_hex = if shard.dasl_cid != 0 {
            format!("0x{:016x}", shard.dasl_cid)
        } else {
            shard.id.split('-').next()
                .filter(|s| s.starts_with("0xda51"))
                .or_else(|| extract_seal_field(&shard, "DASL:").as_deref().map(|_| ""))
                .map(|s| if s.is_empty() {
                    extract_seal_field(&shard, "DASL:").unwrap_or_default()
                } else { s.to_string() })
                .unwrap_or_else(|| "0x0000000000000000".to_string())
        };
        // Extract orbifold from seal header if shard orbifold is zero
        let orbifold = if shard.orbifold == [0, 0, 0] {
            extract_seal_field(&shard, "Orbifold:")
                .unwrap_or_else(|| format!("({},{},{})", shard.orbifold[0], shard.orbifold[1], shard.orbifold[2]))
        } else {
            format!("({},{},{})", shard.orbifold[0], shard.orbifold[1], shard.orbifold[2])
        };
        // Extract witness from seal header
        let witness = extract_seal_field(&shard, "Witness:");
        // Extract URL from seal header
        let url = extract_seal_field(&shard, "URL:");
        // Human name from shard ID
        let human_name = shard.id.split('-').skip(1).collect::<Vec<_>>().join("-");
        let node = format!("shard:{}", shard.id.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_"));
        let bott_idx = (shard.orbifold[2] % 8) as usize;

        // Shard node
        writeln!(out, "{} a erdfa:SheafSection, dasl:Type6 ;", node).unwrap();
        writeln!(out, "    dc:identifier \"{}\" ;", shard.id).unwrap();
        if !human_name.is_empty() {
            writeln!(out, "    dc:title \"{}\" ;", human_name).unwrap();
        }
        writeln!(out, "    erdfa:cid \"{}\" ;", shard.cid).unwrap();
        writeln!(out, "    dasl:addr \"{}\" ;", dasl_hex).unwrap();
        writeln!(out, "    erdfa:shard \"{},{},{}\" ;",
            shard.orbifold[0], shard.orbifold[1], shard.orbifold[2]).unwrap();
        writeln!(out, "    sheaf:orbifold \"{}\" ;", orbifold).unwrap();
        writeln!(out, "    dasl:bott \"{} ({})\" ;", bott_idx, bott_labels[bott_idx]).unwrap();
        writeln!(out, "    erdfa:conjugacyClass {} ;", shard.conjugacy_class).unwrap();
        if let Some(ref w) = witness {
            writeln!(out, "    erdfa:witness \"{}\" ;", w).unwrap();
        }
        if let Some(ref u) = url {
            writeln!(out, "    erdfa:url \"{}\" ;", u).unwrap();
        }

        // Component type + text preview
        emit_component(&mut out, &node, &shard.component, &mut arrow_count);

        // Provenance
        if !shard.provenance.version.is_empty() {
            writeln!(out, "    erdfa:version \"{}\" ;", shard.provenance.version).unwrap();
        }
        if !shard.provenance.git_commit.is_empty() {
            writeln!(out, "    erdfa:gitCommit \"{}\" ;", shard.provenance.git_commit).unwrap();
        }
        if shard.provenance.hash_cell != [0, 0, 0] {
            writeln!(out, "    erdfa:hashCell \"({},{},{})\" ;",
                shard.provenance.hash_cell[0], shard.provenance.hash_cell[1], shard.provenance.hash_cell[2]).unwrap();
        }

        // CFT content extract (clean text from KeyValue content field)
        if let Component::KeyValue { pairs } = &shard.component {
            for (k, v) in pairs {
                if k == "content" && v.len() > 10 {
                    // Extract just the text after the seal header
                    let clean = if let Some(idx) = v.find("---\n") {
                        let after = &v[idx+4..];
                        if let Some(idx2) = after.find("---\n") {
                            after[idx2+4..].trim()
                        } else { after.trim() }
                    } else { v.trim() };
                    let preview: String = clean.chars().take(200).collect();
                    let escaped = preview.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', " ");
                    if !escaped.is_empty() {
                        writeln!(out, "    erdfa:contentPreview \"{}\" ;", escaped).unwrap();
                    }
                }
                if k == "scale" {
                    let scale_name = match v.as_str() {
                        "0" => "cft:post", "1" => "cft:paragraph", "2" => "cft:line",
                        "3" => "cft:token", "4" => "cft:emoji", "5" => "cft:byte",
                        _ => "cft:unknown",
                    };
                    writeln!(out, "    cft:depth {} ;", v).unwrap();
                    writeln!(out, "    cft:scaleName {} ;", scale_name).unwrap();
                }
            }
        }

        // Tags
        if !shard.tags.is_empty() {
            let tags: Vec<String> = shard.tags.iter().map(|t| format!("\"{}\"", t)).collect();
            writeln!(out, "    erdfa:tags {} ;", tags.join(", ")).unwrap();
        }

        writeln!(out, "    .\n").unwrap();
        shard_count += 1;
    }

    // Summary comment
    writeln!(out, "# {} shards, {} arrows", shard_count, arrow_count).unwrap();
    eprintln!("[da51-ttl-erdfa] {} shards, {} arrows", shard_count, arrow_count);
}

fn emit_component(out: &mut dyn Write, parent: &str, comp: &Component, arrows: &mut u64) {
    match comp {
        Component::Paragraph { text } => {
            let preview: String = text.chars().take(80).collect();
            let escaped = preview.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', " ");
            writeln!(out, "    cft:scale cft:paragraph ;").unwrap();
            writeln!(out, "    cft:text \"{}\" ;", escaped).unwrap();
        }
        Component::Heading { level, text } => {
            let escaped = text.replace('\\', "\\\\").replace('"', "\\\"");
            writeln!(out, "    cft:scale cft:heading ;").unwrap();
            writeln!(out, "    cft:level {} ;", level).unwrap();
            writeln!(out, "    cft:text \"{}\" ;", escaped).unwrap();
        }
        Component::Tree { label, children } => {
            let escaped = label.replace('\\', "\\\\").replace('"', "\\\"");
            writeln!(out, "    cft:scale cft:tree ;").unwrap();
            writeln!(out, "    dc:title \"{}\" ;", escaped).unwrap();
            for (i, _child) in children.iter().enumerate() {
                let child_node = format!("{}_c{}", parent, i);
                writeln!(out, "    cft:child {} ;", child_node).unwrap();
                *arrows += 1;
            }
        }
        Component::Group { role, children } => {
            let escaped = role.replace('\\', "\\\\").replace('"', "\\\"");
            writeln!(out, "    cft:scale cft:group ;").unwrap();
            writeln!(out, "    cft:role \"{}\" ;", escaped).unwrap();
            for (i, _child) in children.iter().enumerate() {
                let child_node = format!("{}_c{}", parent, i);
                writeln!(out, "    cft:child {} ;", child_node).unwrap();
                *arrows += 1;
            }
        }
        Component::Code { language, source } => {
            let preview: String = source.chars().take(80).collect();
            let escaped = preview.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', " ");
            writeln!(out, "    cft:scale cft:code ;").unwrap();
            writeln!(out, "    cft:language \"{}\" ;", language).unwrap();
            writeln!(out, "    cft:text \"{}\" ;", escaped).unwrap();
        }
        Component::Link { href, label } => {
            writeln!(out, "    cft:scale cft:link ;").unwrap();
            writeln!(out, "    cft:href \"{}\" ;", href).unwrap();
            writeln!(out, "    cft:label \"{}\" ;", label).unwrap();
        }
        Component::KeyValue { pairs } => {
            writeln!(out, "    cft:scale cft:keyvalue ;").unwrap();
            for (k, v) in pairs {
                let ek = k.replace('"', "\\\"");
                let ev = v.replace('"', "\\\"");
                writeln!(out, "    cft:entry [ cft:key \"{}\"; cft:value \"{}\" ] ;", ek, ev).unwrap();
            }
        }
        _ => {
            writeln!(out, "    cft:scale cft:other ;").unwrap();
        }
    }
}

// cft — Conformal Field Tower decomposition
// Decomposes text into scale layers: Post → Paragraph → Line → Token → Emoji → Byte
// Each layer has n-grams. Arrows between layers are parent→child shard refs.

use crate::{Shard, Component};

/// Scale layers of the conformal tower, coarse → fine
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scale {
    Post,       // whole document
    Paragraph,  // \n\n separated
    Line,       // \n separated
    Token,      // whitespace/punct split
    Emoji,      // unicode emoji sequences
    Byte,       // raw bytes
}

impl Scale {
    pub fn tag(&self) -> &'static str {
        match self {
            Scale::Post => "cft.post",
            Scale::Paragraph => "cft.paragraph",
            Scale::Line => "cft.line",
            Scale::Token => "cft.token",
            Scale::Emoji => "cft.emoji",
            Scale::Byte => "cft.byte",
        }
    }
    pub fn depth(&self) -> u8 {
        match self {
            Scale::Post => 0,
            Scale::Paragraph => 1,
            Scale::Line => 2,
            Scale::Token => 3,
            Scale::Emoji => 4,
            Scale::Byte => 5,
        }
    }
}

/// One node in the conformal tower
pub struct FieldNode {
    pub scale: Scale,
    pub index: usize,       // position within parent
    pub content: String,
    pub parent_id: Option<String>,
}

/// Arrow between layers (scale morphism)
pub struct Arrow {
    pub from: String,  // parent shard id
    pub to: String,    // child shard id
    pub scale_from: Scale,
    pub scale_to: Scale,
}

/// N-gram at a given scale
fn ngrams(items: &[&str], n: usize) -> Vec<String> {
    if items.len() < n { return vec![]; }
    items.windows(n).map(|w| w.join(" ")).collect()
}

/// Decompose text into the full conformal tower.
/// Returns (shards, arrows) — shards are DA51-ready, arrows are KeyValue shards.
/// max_depth: 0=post only, 1=+paragraph, 2=+line, 3=+token, 4=+emoji, 5=+byte
pub fn decompose(id_prefix: &str, text: &str) -> (Vec<Shard>, Vec<Shard>) {
    decompose_depth(id_prefix, text, 5)
}

pub fn decompose_depth(id_prefix: &str, text: &str, max_depth: u8) -> (Vec<Shard>, Vec<Shard>) {
    let mut shards = Vec::new();
    let mut arrows = Vec::new();

    // Post level
    let post_id = format!("{}_post", id_prefix);
    let post_tokens: Vec<&str> = text.split_whitespace().collect();
    shards.push(field_shard(&post_id, Scale::Post, 0, text, None, &post_tokens));

    if max_depth < 1 { return (shards, arrows); }

    // Paragraph level
    let paragraphs: Vec<&str> = text.split("\n\n").filter(|s| !s.trim().is_empty()).collect();
    for (i, para) in paragraphs.iter().enumerate() {
        let pid = format!("{}_p{}", id_prefix, i);
        let toks: Vec<&str> = para.split_whitespace().collect();
        shards.push(field_shard(&pid, Scale::Paragraph, i, para, Some(&post_id), &toks));
        arrows.push(arrow_shard(&post_id, &pid, Scale::Post, Scale::Paragraph));

        if max_depth < 2 { continue; }

        // Line level
        let lines: Vec<&str> = para.lines().filter(|l| !l.trim().is_empty()).collect();
        for (j, line) in lines.iter().enumerate() {
            let lid = format!("{}_p{}_l{}", id_prefix, i, j);
            let ltoks: Vec<&str> = line.split_whitespace().collect();
            shards.push(field_shard(&lid, Scale::Line, j, line, Some(&pid), &ltoks));
            arrows.push(arrow_shard(&pid, &lid, Scale::Paragraph, Scale::Line));

            if max_depth < 3 { continue; }

            // Token level
            let tokens: Vec<&str> = line.split_whitespace().collect();
            for (k, tok) in tokens.iter().enumerate() {
                let tid = format!("{}_p{}_l{}_t{}", id_prefix, i, j, k);
                shards.push(field_shard(&tid, Scale::Token, k, tok, Some(&lid), &[]));
                arrows.push(arrow_shard(&lid, &tid, Scale::Line, Scale::Token));

                if max_depth < 4 { continue; }

                // Emoji level — extract emoji codepoints
                let emojis: Vec<String> = tok.chars()
                    .filter(|c| is_emoji(*c))
                    .map(|c| format!("U+{:04X}", c as u32))
                    .collect();
                if !emojis.is_empty() {
                    let eid = format!("{}_p{}_l{}_t{}_e", id_prefix, i, j, k);
                    shards.push(Shard::new(
                        &eid,
                        Component::List { ordered: true, items: emojis },
                    ).with_tags(vec!["cft".into(), Scale::Emoji.tag().into(), format!("parent:{}", tid)]));
                    arrows.push(arrow_shard(&tid, &eid, Scale::Token, Scale::Emoji));
                }

                if max_depth < 5 { continue; }

                // Byte level
                let bytes: Vec<String> = tok.bytes().map(|b| format!("{:02x}", b)).collect();
                let bid = format!("{}_p{}_l{}_t{}_b", id_prefix, i, j, k);
                shards.push(Shard::new(
                    &bid,
                    Component::Code { language: "hex".into(), source: bytes.join(" ") },
                ).with_tags(vec!["cft".into(), Scale::Byte.tag().into(), format!("parent:{}", tid)]));
                arrows.push(arrow_shard(&tid, &bid, Scale::Token, Scale::Byte));
            }
        }
    }

    (shards, arrows)
}

fn field_shard(id: &str, scale: Scale, index: usize, content: &str, parent: Option<&str>, tokens: &[&str]) -> Shard {
    let mut pairs = vec![
        ("scale".into(), format!("{}", scale.depth())),
        ("index".into(), index.to_string()),
        ("len".into(), content.len().to_string()),
    ];
    if let Some(p) = parent {
        pairs.push(("parent".into(), p.into()));
    }
    // N-grams: bigrams and trigrams at this scale
    if tokens.len() >= 2 {
        let bi = ngrams(tokens, 2);
        pairs.push(("bigrams".into(), bi.join(" | ")));
    }
    if tokens.len() >= 3 {
        let tri = ngrams(tokens, 3);
        pairs.push(("trigrams".into(), tri.join(" | ")));
    }
    pairs.push(("content".into(), truncate(content, 512)));

    Shard::new(id, Component::KeyValue { pairs })
        .with_tags(vec!["cft".into(), scale.tag().into()])
}

fn arrow_shard(from: &str, to: &str, sf: Scale, st: Scale) -> Shard {
    Shard::new(
        &format!("{}→{}", from, to),
        Component::KeyValue {
            pairs: vec![
                ("from".into(), from.into()),
                ("to".into(), to.into()),
                ("scale_from".into(), sf.depth().to_string()),
                ("scale_to".into(), st.depth().to_string()),
                ("morphism".into(), format!("{}→{}", sf.tag(), st.tag())),
            ],
        },
    ).with_tags(vec!["cft".into(), "arrow".into()])
}

fn is_emoji(c: char) -> bool {
    let cp = c as u32;
    // Common emoji ranges
    (0x1F600..=0x1F64F).contains(&cp) ||  // emoticons
    (0x1F300..=0x1F5FF).contains(&cp) ||  // symbols & pictographs
    (0x1F680..=0x1F6FF).contains(&cp) ||  // transport
    (0x1F900..=0x1F9FF).contains(&cp) ||  // supplemental
    (0x2600..=0x26FF).contains(&cp) ||    // misc symbols
    (0x2700..=0x27BF).contains(&cp) ||    // dingbats
    (0xFE00..=0xFE0F).contains(&cp) ||    // variation selectors
    (0x200D..=0x200D).contains(&cp)       // ZWJ
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { return s.to_string(); }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) { end -= 1; }
    format!("{}…", &s[..end])
}

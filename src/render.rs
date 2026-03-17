use crate::{Component, Shard};

/// Render a shard's component as plain text
pub fn render_text(shard: &Shard) -> String {
    let mut out = String::new();
    out.push_str(&format!("── {} ── [{}]\n", shard.id, shard.tags.join(", ")));
    render_component_text(&shard.component, &mut out, 0);
    out
}

/// Render a shard's component as HTML fragment
pub fn render_html(shard: &Shard) -> String {
    let mut out = String::new();
    out.push_str(&format!("<article data-cid=\"{}\" data-tags=\"{}\">\n",
        shard.cid, shard.tags.join(","))); 
    render_component_html(&shard.component, &mut out);
    out.push_str("</article>\n");
    out
}

fn render_component_text(c: &Component, out: &mut String, indent: usize) {
    let pad: String = "  ".repeat(indent);
    match c {
        Component::Heading { level, text } => {
            let marker: String = "#".repeat(*level as usize);
            out.push_str(&format!("{}{} {}\n", pad, marker, text));
        }
        Component::Paragraph { text } => {
            out.push_str(&format!("{}{}\n", pad, text));
        }
        Component::Code { language, source } => {
            out.push_str(&format!("{}```{}\n{}{}\n{}```\n", pad, language, pad, source, pad));
        }
        Component::Table { headers, rows } => {
            out.push_str(&format!("{}| {} |\n", pad, headers.join(" | ")));
            out.push_str(&format!("{}|{}|\n", pad, headers.iter().map(|h| "-".repeat(h.len() + 2)).collect::<Vec<_>>().join("|")));
            for row in rows {
                out.push_str(&format!("{}| {} |\n", pad, row.join(" | ")));
            }
        }
        Component::List { ordered, items } => {
            for (i, item) in items.iter().enumerate() {
                if *ordered { out.push_str(&format!("{}{:2}. {}\n", pad, i + 1, item)); }
                else { out.push_str(&format!("{}  • {}\n", pad, item)); }
            }
        }
        Component::KeyValue { pairs } => {
            for (k, v) in pairs {
                out.push_str(&format!("{}{}: {}\n", pad, k, v));
            }
        }
        Component::MapEntity { name, kind, x, y, meta } => {
            out.push_str(&format!("{}◆ {} [{}] at ({:.0},{:.0})\n", pad, name, kind, x, y));
            for (k, v) in meta {
                out.push_str(&format!("{}  {}: {}\n", pad, k, v));
            }
        }
        Component::Tree { label, children } => {
            out.push_str(&format!("{}├─ {}\n", pad, label));
            for child in children {
                render_component_text(child, out, indent + 1);
            }
        }
        Component::Link { href, label } => {
            out.push_str(&format!("{}[{}]({})\n", pad, label, href));
        }
        Component::Image { alt, cid } => {
            out.push_str(&format!("{}![{}](ipfs://{})\n", pad, alt, cid));
        }
        Component::Group { role, children } => {
            out.push_str(&format!("{}── {} ──\n", pad, role));
            for child in children {
                render_component_text(child, out, indent + 1);
            }
        }
    }
}

fn render_component_html(c: &Component, out: &mut String) {
    match c {
        Component::Heading { level, text } => {
            out.push_str(&format!("<h{0}>{1}</h{0}>\n", level, text));
        }
        Component::Paragraph { text } => {
            out.push_str(&format!("<p>{}</p>\n", text));
        }
        Component::Code { language, source } => {
            out.push_str(&format!("<pre><code class=\"language-{}\">{}</code></pre>\n", language, source));
        }
        Component::Table { headers, rows } => {
            out.push_str("<table>\n<thead><tr>");
            for h in headers { out.push_str(&format!("<th>{}</th>", h)); }
            out.push_str("</tr></thead>\n<tbody>\n");
            for row in rows {
                out.push_str("<tr>");
                for cell in row { out.push_str(&format!("<td>{}</td>", cell)); }
                out.push_str("</tr>\n");
            }
            out.push_str("</tbody></table>\n");
        }
        Component::List { ordered, items } => {
            let tag = if *ordered { "ol" } else { "ul" };
            out.push_str(&format!("<{}>\n", tag));
            for item in items { out.push_str(&format!("<li>{}</li>\n", item)); }
            out.push_str(&format!("</{}>\n", tag));
        }
        Component::KeyValue { pairs } => {
            out.push_str("<dl>\n");
            for (k, v) in pairs { out.push_str(&format!("<dt>{}</dt><dd>{}</dd>\n", k, v)); }
            out.push_str("</dl>\n");
        }
        Component::MapEntity { name, kind, x, y, meta } => {
            out.push_str(&format!("<div class=\"map-entity\" data-kind=\"{}\" style=\"left:{:.0}px;top:{:.0}px\">\n", kind, x, y));
            out.push_str(&format!("  <span class=\"entity-name\">{}</span>\n", name));
            for (k, v) in meta { out.push_str(&format!("  <span class=\"meta\" data-key=\"{}\">{}</span>\n", k, v)); }
            out.push_str("</div>\n");
        }
        Component::Tree { label, children } => {
            out.push_str(&format!("<details><summary>{}</summary>\n", label));
            for child in children { render_component_html(child, out); }
            out.push_str("</details>\n");
        }
        Component::Link { href, label } => {
            out.push_str(&format!("<a href=\"{}\">{}</a>\n", href, label));
        }
        Component::Image { alt, cid } => {
            out.push_str(&format!("<img alt=\"{}\" src=\"https://ipfs.io/ipfs/{}\">\n", alt, cid));
        }
        Component::Group { role, children } => {
            out.push_str(&format!("<section role=\"{}\">\n", role));
            for child in children { render_component_html(child, out); }
            out.push_str("</section>\n");
        }
    }
}

/// Decode a DA51-tagged CBOR shard from bytes
pub fn decode_shard(bytes: &[u8]) -> Option<Shard> {
    let value: ciborium::Value = ciborium::from_reader(bytes).ok()?;
    if let ciborium::Value::Tag(55889, inner) = value {
        ciborium::value::Value::deserialized(&inner).ok()
    } else {
        None
    }
}

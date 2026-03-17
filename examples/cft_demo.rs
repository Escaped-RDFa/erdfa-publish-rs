use erdfa_publish::cft;

fn main() {
    let text = "Hello world 🌍\n\nThis is a test paragraph.\nWith two lines.\n\nThird paragraph has emoji 🚀✨ and bytes.";

    let (shards, arrows) = cft::decompose("demo", text);

    println!("=== SHARDS ({}) ===", shards.len());
    for s in &shards {
        let tags: String = s.tags.join(", ");
        println!("  {} [{}] ({}B cbor)", s.id, tags, s.to_cbor().len());
    }

    println!("\n=== ARROWS ({}) ===", arrows.len());
    for a in &arrows {
        println!("  {} [{}]", a.id, a.tags.join(", "));
    }

    println!("\n=== TOTAL: {} shards + {} arrows = {} DA51 objects ===",
        shards.len(), arrows.len(), shards.len() + arrows.len());

    // Verify CBOR
    let cbor = shards[0].to_cbor();
    println!("\nFirst shard CBOR: {}B", cbor.len());
}

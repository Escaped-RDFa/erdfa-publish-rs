use erdfa_publish::*;

fn main() {
    let mut set = ShardSet::new("shem-72-demo");

    let s1 = Shard::new("heading-1", Component::Heading {
        level: 1, text: "72 Names of God".into(),
    }).with_tags(vec!["monster".into(), "kabbalah".into()]);

    let s2 = Shard::new("table-basins", Component::Table {
        headers: vec!["Basin".into(), "Class".into(), "Count".into()],
        rows: vec![
            vec!["B0".into(), "A".into(), "12".into()],
            vec!["B1".into(), "AIII".into(), "8".into()],
        ],
    });

    let s3 = Shard::new("map-zion", Component::MapEntity {
        name: "Zion".into(), kind: "PLACE".into(),
        x: 14.0, y: 7.0,
        meta: vec![("blade".into(), "e{1,2,3}".into())],
    });

    set.add(&s1);
    set.add(&s2);
    set.add(&s3);

    let shards = vec![s1.clone(), s2.clone(), s3.clone()];
    let mut tar = std::fs::File::create("/tmp/erdfa-demo.tar").unwrap();
    set.to_tar(&shards, &mut tar).unwrap();

    let cbor = set.to_cbor();
    println!("manifest: {}B CBOR, {} shards", cbor.len(), set.shards.len());
    for s in &set.shards {
        println!("  {} cid={} tags={:?}", s.id, s.cid, s.tags);
    }
    println!("tar: /tmp/erdfa-demo.tar ({}B)", std::fs::metadata("/tmp/erdfa-demo.tar").unwrap().len());

    // Emit a richer promoted manifest alongside the thin ShardSet manifest.
    let mut promoted = PromotedShardSet::new(
        "shem-72-demo",
        "shem-72-demo",
        "rev-demo",
        "erdfa-shard-set",
        "2026-03-29T00:00:00Z",
    )
    .with_build_provenance(BuildProvenance {
        builder: "erdfa-publish-rs".into(),
        source_repo: Some("https://github.com/meta-introspector/erdfa-publish".into()),
        source_revision: Some("demo".into()),
        build_command: Some("cargo run --example demo".into()),
    });
    promoted.add_ref(s1.promoted_ref().with_logical_kind("heading"));
    promoted.add_ref(s2.promoted_ref().with_logical_kind("table"));
    promoted.add_ref(s3.promoted_ref().with_logical_kind("map"));

    let promoted_cbor = promoted.to_cbor();
    std::fs::write("/tmp/erdfa-promoted-manifest.cbor", &promoted_cbor).unwrap();
    std::fs::write(
        "/tmp/erdfa-promoted-manifest.json",
        serde_json::to_vec_pretty(&promoted).unwrap(),
    )
    .unwrap();
    println!(
        "promoted manifest: {}B CBOR, {} shards (paths: /tmp/erdfa-promoted-manifest.cbor, /tmp/erdfa-promoted-manifest.json)",
        promoted_cbor.len(),
        promoted.shards.len()
    );
}

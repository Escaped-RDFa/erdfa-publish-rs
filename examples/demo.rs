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

    let mut tar = std::fs::File::create("/tmp/erdfa-demo.tar").unwrap();
    set.to_tar(&[s1, s2, s3], &mut tar).unwrap();

    let cbor = set.to_cbor();
    println!("manifest: {}B CBOR, {} shards", cbor.len(), set.shards.len());
    for s in &set.shards {
        println!("  {} cid={} tags={:?}", s.id, s.cid, s.tags);
    }
    println!("tar: /tmp/erdfa-demo.tar ({}B)", std::fs::metadata("/tmp/erdfa-demo.tar").unwrap().len());
}

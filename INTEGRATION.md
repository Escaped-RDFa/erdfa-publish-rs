# Integrating erdfa-publish into your Rust project

## 1. Add the dependency

```toml
# Cargo.toml
[dependencies]
erdfa-publish = { path = "/path/to/erdfa-publish" }
```

Or from the local git mirror:

```toml
erdfa-publish = { git = "https://github.com/meta-introspector/erdfa-publish" }
```

## 2. Import

```rust
use erdfa_publish::{Component, Shard, ShardSet};
```

## 3. Create shards

Each shard wraps a semantic `Component`. The CID is computed automatically from the component content (SHA-256, bafk-prefixed).

```rust
// Heading
let s1 = Shard::new("intro", Component::Heading {
    level: 1,
    text: "72 Names of God".into(),
});

// Key-value pairs
let s2 = Shard::new("accumulator", Component::KeyValue {
    pairs: vec![
        ("blade".into(), "e[1,2,3,4,7,8]".into()),
        ("SSP".into(), "{3,5,7,11,19,23}".into()),
        ("grade".into(), "6".into()),
    ],
});

// Map entity (for location maps)
let s3 = Shard::new("entity-exodus", Component::MapEntity {
    name: "exodus".into(),
    kind: "GOD".into(),
    x: 23.0, y: 13.0,
    meta: vec![
        ("blade".into(), "0b110011110".into()),
        ("egyptian".into(), "false".into()),
    ],
});

// Code block
let s4 = Shard::new("source", Component::Code {
    language: "rust".into(),
    source: "let acc = e1.gp(&e2);".into(),
});

// Table (e.g., confusion matrix)
let s5 = Shard::new("confusion", Component::Table {
    headers: vec!["".into(), "GOD".into(), "PERS".into(), "THNG".into()],
    rows: vec![
        vec!["GOD".into(), "4".into(), "1".into(), "0".into()],
        vec!["PERS".into(), "0".into(), "3".into(), "1".into()],
    ],
});

// Tags for filtering
let s2 = s2.with_tags(vec!["clifford".into(), "accumulator".into()]);
```

## 4. Available components

| Component    | Fields                                      | Use case                        |
|-------------|---------------------------------------------|---------------------------------|
| `Heading`    | `level: u8, text: String`                   | Section headers                 |
| `Paragraph`  | `text: String`                              | Prose, descriptions             |
| `Code`       | `language: String, source: String`          | Source code, formulas           |
| `Table`      | `headers: Vec<String>, rows: Vec<Vec<String>>` | Matrices, data tables        |
| `Tree`       | `label: String, children: Vec<Component>`   | Hierarchical data               |
| `List`       | `ordered: bool, items: Vec<String>`         | Bullet/numbered lists           |
| `Link`       | `href: String, label: String`               | External references             |
| `Image`      | `alt: String, cid: String`                  | Images by CID                   |
| `KeyValue`   | `pairs: Vec<(String, String)>`              | Metadata, properties            |
| `MapEntity`  | `name, kind: String, x, y: f64, meta: Vec<(String, String)>` | Location map entities |
| `Group`      | `role: String, children: Vec<Component>`    | Semantic grouping               |

## 5. Build a manifest and tar archive

```rust
let shards = vec![s1, s2, s3, s4, s5];

let manifest = ShardSet::from_shards("72-names", &shards);

// Write tar archive (manifest.cbor + per-shard .cbor files)
let mut file = std::fs::File::create("output.tar").unwrap();
manifest.to_tar(&shards, &mut file).unwrap();
```

The tar contains:
```
intro.cbor          # DA51-tagged CBOR shard
accumulator.cbor
entity-exodus.cbor
source.cbor
confusion.cbor
manifest.cbor       # ShardSet with all ShardRefs
```

## 6. CBOR format

Every shard is wrapped in a CBOR tag `55889` (0xDA51):

```
Tag(55889, {
    "id": "accumulator",
    "cid": "bafk...",
    "component": { "type": "KeyValue", "pairs": [...] },
    "tags": ["clifford", "accumulator"]
})
```

Wire-compatible with the eRDFa WASM renderer.

## 7. URL helpers

```rust
shard.ipfs_url()   // https://ipfs.io/ipfs/bafk...
shard.paste_url("https://solana.solfunmeme.com/pastebin")
                   // https://solana.solfunmeme.com/pastebin/raw/accumulator
```

## 8. Example: shem-hamephorash-72 integration

```rust
use erdfa_publish::{Component, Shard, ShardSet};

// At end of main(), after all computation:
let mut shards = Vec::new();

// Export each group as a MapEntity shard
for (i, (loc, _, blade)) in all_groups.iter().enumerate() {
    let bk = blade.dominant_key();
    shards.push(Shard::new(
        format!("group-{}", i),
        Component::MapEntity {
            name: loc.clone(),
            kind: class_names[labels[i] as usize].into(),
            x: (bk & 0xFF) as f64,
            y: ((bk >> 4) & 0xFF) as f64,
            meta: vec![
                ("blade".into(), format!("{:#b}", bk)),
                ("grade".into(), format!("{}", bk.count_ones())),
                ("egyptian".into(), format!("{}", has_egyptian_loanwords(i))),
            ],
        },
    ));
}

// Export accumulator
shards.push(Shard::new("accumulator", Component::KeyValue {
    pairs: vec![
        ("blade".into(), "e[1,2,3,4,7,8]".into()),
        ("coefficient".into(), "-1.0".into()),
        ("SSP".into(), "{3,5,7,11,19,23}".into()),
    ],
}));

let manifest = ShardSet::from_shards("shem-hamephorash-72", &shards);
let mut f = std::fs::File::create("shem72.tar").unwrap();
manifest.to_tar(&shards, &mut f).unwrap();
```

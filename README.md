# erdfa-publish

Semantic UI components as CBOR shards. Define structure in Rust, render anywhere.

Includes a **Conformal Field Tower (CFT)** module that decomposes any text into multi-scale layers — post, paragraph, line, token, emoji, bytes — with n-grams and typed arrows between layers. Every node and edge is a content-addressed DA51 CBOR shard.

## Concept

Instead of writing HTML/JS, you describe **what** your UI contains — headings, tables, trees, maps, code blocks — as typed Rust structs. These get serialized as CBOR shards with content-addressed IDs. Any renderer (browser, screen reader, CLI, embedded display) loads the shards and presents them according to its own a11y layer and CSS.

```
Rust program → Component structs → CBOR shards → loader → renderer
                                                          ├─ visual CSS
                                                          ├─ screen reader
                                                          ├─ CLI table
                                                          └─ braille display
```

## Install

```toml
[dependencies]
erdfa-publish = { git = "https://github.com/meta-introspector/erdfa-publish" }
```

## Quick start

```rust
use erdfa_publish::*;

// Create semantic components
let heading = Component::Heading { level: 1, text: "Results".into() };
let table = Component::Table {
    headers: vec!["Name".into(), "Value".into()],
    rows: vec![vec!["alpha".into(), "0.73".into()]],
};

// Wrap as shards (auto-generates CID from content hash)
let s1 = Shard::new("result-heading", heading);
let s2 = Shard::new("result-table", table).with_tags(vec!["data".into()]);

// Build manifest + tar archive
let mut set = ShardSet::new("my-results");
set.add(&s1);
set.add(&s2);
set.to_tar(&[s1, s2], std::fs::File::create("output.tar").unwrap()).unwrap();
```

## Conformal Field Tower (CFT)

Decompose any text into a tower of scale layers. Each layer is a shard, each edge is an arrow shard. N-grams (bigrams, trigrams) are computed at each level.

```
Scale 0: Post          "Hello world 🌍\n\nSecond paragraph."
  │                     bigrams: "Hello world" | "world 🌍" | ...
  ├─→ Scale 1: Paragraph₀   "Hello world 🌍"
  │     ├─→ Scale 2: Line₀       "Hello world 🌍"
  │     │     ├─→ Scale 3: Token₀    "Hello"
  │     │     │     └─→ Scale 5: Byte   "48 65 6c 6c 6f"
  │     │     ├─→ Scale 3: Token₁    "world"
  │     │     │     └─→ Scale 5: Byte   "77 6f 72 6c 64"
  │     │     └─→ Scale 3: Token₂    "🌍"
  │     │           ├─→ Scale 4: Emoji  [U+1F30D]
  │     │           └─→ Scale 5: Byte   "f0 9f 8c 8d"
  └─→ Scale 1: Paragraph₁   ...
```

### Usage

```rust
use erdfa_publish::cft;

let text = "Hello world 🌍\n\nThis is a test paragraph.\nWith two lines.";
let (shards, arrows) = cft::decompose("my-doc", text);

// shards: field nodes at every scale (Post, Paragraph, Line, Token, Emoji, Byte)
// arrows: typed edges between layers (parent→child with scale metadata)

// Every object is a DA51 CBOR shard
for shard in &shards {
    std::fs::write(
        format!("{}.cbor", shard.id),
        shard.to_cbor(),
    ).unwrap();
}
```

### Scale layers

| Scale | Depth | Splits on | N-grams | Component type |
|-------|-------|-----------|---------|---------------|
| Post | 0 | — | bigrams, trigrams of all tokens | KeyValue |
| Paragraph | 1 | `\n\n` | bigrams, trigrams | KeyValue |
| Line | 2 | `\n` | bigrams, trigrams | KeyValue |
| Token | 3 | whitespace | — | KeyValue |
| Emoji | 4 | unicode ranges | — | List (codepoints) |
| Byte | 5 | — | — | Code (hex) |

### Arrow shards

Every parent→child relationship is itself a shard:

```
DA51 tag → {
  "id": "my-doc_post→my-doc_p0",
  "component": {
    "type": "KeyValue",
    "pairs": [
      ["from", "my-doc_post"],
      ["to", "my-doc_p0"],
      ["scale_from", "0"],
      ["scale_to", "1"],
      ["morphism", "cft.post→cft.paragraph"]
    ]
  },
  "tags": ["cft", "arrow"]
}
```

### Scale as a functor

The decomposition is a functor from the category of texts to the category of shard diagrams. Each scale transformation (post→paragraph, paragraph→line, etc.) is a natural transformation. The arrows are morphisms. The n-grams are local invariants preserved across scales.

## Component types

| Type | Fields | Semantic meaning |
|------|--------|-----------------|
| `Heading` | `level`, `text` | Section header (1–6) |
| `Paragraph` | `text` | Block of prose |
| `Code` | `language`, `source` | Source code with syntax hint |
| `Table` | `headers`, `rows` | Tabular data |
| `Tree` | `label`, `children` | Recursive hierarchy |
| `List` | `ordered`, `items` | Ordered or unordered list |
| `Link` | `href`, `label` | Navigation reference |
| `Image` | `alt`, `cid` | Image by content address |
| `KeyValue` | `pairs` | Metadata / properties |
| `MapEntity` | `name`, `kind`, `x`, `y`, `meta` | Positioned entity on a map |
| `Group` | `role`, `children` | Container with semantic role |

## CBOR format

Every shard and manifest is wrapped in CBOR tag **55889** (`0xDA51`):

```
DA51 tag → {
  "id": "result-table",
  "cid": "bafk205260a6c670b02f...",
  "component": { "type": "Table", "headers": [...], "rows": [...] },
  "tags": ["data"]
}
```

## Tar archive layout

```
output.tar
├── result-heading.cbor    # DA51-tagged shard
├── result-table.cbor      # DA51-tagged shard
└── manifest.cbor          # DA51-tagged ShardSet
```

## Promoted manifests

`ShardSet` stays as the thin manifest for existing shard bundles. `PromotedShardSet` adds the richer catalog fields needed when a publisher needs artifact provenance, per-shard encoding/size metadata, sink refs, and routing hints without changing shard payloads.

```rust
use erdfa_publish::*;

let shard = Shard::new("left-001", Component::Paragraph { text: "hello".into() })
    .with_tags(vec!["demo".into()]);

let promoted = shard
    .promoted_ref()
    .with_logical_kind("route-bucket")
    .with_object_refs(vec![ObjectRef {
        sink: "hf".into(),
        uri: "hf://datasets/demo/left-001.cbor".into(),
        size_bytes: 128,
        content_digest: "sha256:deadbeef".into(),
    }])
    .with_routing_keys(vec!["route-left-node=Q123".into()]);

let mut manifest = PromotedShardSet::new(
    "demo",
    "artifact-demo",
    "rev-a",
    "erdfa-shard-set",
    "2026-03-29T00:00:00Z",
);
manifest.add_ref(promoted);
```

## Local publish workflow

The publish substrate now includes an additive local workflow that keeps shard
identity stable while attaching sink refs and receipts.

```rust
use erdfa_publish::{
    apply_sink_adapters, build_local_artifact_bundle, Component, FileSinkAdapter,
    HfSinkAdapter, HostedAcknowledgement, IpfsSinkAdapter, Shard,
};

let shards = vec![
    Shard::new("left-bucket-001", Component::Paragraph { text: "left".into() }),
    Shard::new("nodeOfName-en-017", Component::Paragraph { text: "tenant".into() }),
];

let bundle = build_local_artifact_bundle("artifact-demo", "rev-a", &shards);

let file = FileSinkAdapter { output_root: std::env::temp_dir().join("erdfa-publish-local") };
let hf = HfSinkAdapter { dataset_root: "hf://datasets/example/zelph".into() };
let ipfs = IpfsSinkAdapter;

let mut outcomes = apply_sink_adapters(&bundle, &[&file, &hf, &ipfs]).unwrap();
let hf_receipt = HostedAcknowledgement {
    sink: "hf".into(),
    acknowledgement_id: "commit:deadbeef".into(),
    locator_uri: outcomes["hf"].receipt.container_ref.uri.clone(),
    verification_url: Some("https://huggingface.co/datasets/example/zelph/commit/deadbeef".into()),
    content_digest: outcomes["hf"].receipt.container_ref.content_digest.clone(),
    size_bytes: outcomes["hf"].receipt.container_ref.size_bytes,
    verified: true,
};
let hf_outcome = outcomes.remove("hf").unwrap().bind_hosted_acknowledgement(hf_receipt).unwrap();
assert!(hf_outcome.receipt.hosted_acknowledgement.is_some());
assert!(outcomes["file"].receipt.sink_refs.len() == shards.len());
```

The workflow emits:

- one promoted manifest with per-shard object refs
- one container index with member paths and content digests
- one receipt per sink containing artifact id/revision, shard refs, sink refs,
  container ref, content digests, publish result, and optional hosted
  acknowledgement

HF and IPFS adapters in this crate are local-first projections that attach
stable sink refs and receipts. They do not mutate shard semantics or claim
remote publish acknowledgement by themselves. When an external uploader verifies
an acknowledged commit or CID against the projected container ref, that hosted
acknowledgement can be bound back into the native `PublishReceipt`.

## Hosted publish workflow

Under the native feature, the crate now also exposes bounded hosted helpers:

- `publish_hf_with_ack(...)`
- `publish_ipfs_with_ack(...)`

These helpers:

- project the normal publish outcome first
- perform a live hosted upload and read-back verification
- bind the resulting hosted acknowledgement back into the native
  `PublishReceipt`
- write first-class emitted artifacts for each sink:
  - `manifest.json`
  - `container-index.json`
  - `receipt.json`

Example:

```bash
ERDFA_HF_DATASET_ROOT=hf://datasets/chbwa/itir-zos-ack-probe \
ERDFA_IPFS_API=http://127.0.0.1:5001 \
ERDFA_IPFS_GATEWAY=http://127.0.0.1:8080 \
ERDFA_PUBLISH_OUTPUT_ROOT=/tmp/erdfa-publish-hosted \
cargo run --example publish_hosted
```

Current limitation:

- HF hosted acknowledgement binds cleanly against the projected container URI
- IPFS hosted acknowledgement currently binds at the container level only; the
  per-shard IPFS refs remain projected values until the adapter is refactored
  to emit real hosted shard refs instead of synthetic local `content_cid(...)`
  placeholders

Current blocker for always-on network integration tests: this repo does not own
an unauthenticated public write surface for HF/IPFS, and gateway reachability
is not deterministic in CI. The crate therefore validates deterministic local
publish contracts and keeps remote acknowledgement out of scope.

## Rendering

Shards are semantic, not visual. A loader fetches shards by CID, reads the `type` field, and delegates to the active a11y layer:

- **Visual**: CSS grid, syntax highlighting, interactive maps
- **Screen reader**: ARIA roles derived from component type
- **CLI**: ASCII tables, indented trees, plain text
- **Minimal**: progressive loading — show N/total progress

The `Group` component with a `role` field maps directly to ARIA landmarks (`navigation`, `main`, `complementary`, etc.).

## URLs

```rust
let shard = Shard::new("my-data", component);
shard.ipfs_url()                    // https://ipfs.io/ipfs/bafk...
shard.paste_url("http://host:8090") // http://host:8090/raw/my-data
```

## Tools

### `tools/dasl_reader.py` — Python DASL/CBOR reader

A standalone Python tool for reading DA51-tagged CBOR shards and parsing 64-bit DASL addresses.

Requires: `pip install cbor2` (or use the nix flake from [neural-moonshine](https://github.com/fargolo/neural-moonshine))

```bash
# Parse a DASL address
python3 tools/dasl_reader.py addr 0xDA510001F9080000

# Read a CBOR shard
python3 tools/dasl_reader.py read shards/note1.cbor

# Scan a directory for all shards
python3 tools/dasl_reader.py scan shards/

# Export shard content as binary
python3 tools/dasl_reader.py export shards/ output.bin
```

Supports all DASL address types: MonsterWalk, ASTNode, Protocol, NestedCID, HarmonicPath, ShardID, Eigenspace, Hauptmodul.

## License

MIT OR Apache-2.0

## Customer Onboarding

**New here?** See [docs/CUSTOMER_ONBOARDING.md](docs/CUSTOMER_ONBOARDING.md) — run your first Cl(15,0,0) experiment and get AI peer review in 3 minutes.

Any user who can produce valid DA51-tagged CBOR shards is a customer. The `erdfa-cli` tool handles the encoding — you bring the experiment.

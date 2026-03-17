# Integrating erdfa-publish into mmgroup-rust

## Goal

Export mmgroup-rust analysis outputs (basin discovery, FRACTRAN, Clifford algebra, DASL, Leech lattice, etc.) as semantic CBOR shards. The VM loads and renders them per a11y layer.

## Setup

Add to `Cargo.toml`:

```toml
erdfa-publish = { path = "../erdfa-publish" }
```

## Mapping: existing bins → shard types

| Binary | Output | Component |
|---|---|---|
| `basin_discovery` | basin classification results | `Table` (basin, class, count) |
| `basin_fractran_verify` | proof verification | `KeyValue` (theorem, status, witness) |
| `basin_gui` / `monster_tui` | TUI screens | `Group { role: "screen" }` with children |
| `dasl_*` | DASL chain/broadcast/walk | `Tree` (chain steps) + `KeyValue` (state) |
| `clifford_roundtrip_demo` | Cl(15,0,0) roundtrip | `Table` (blade, value, grade) |
| `fractran_clifford_run` | FRACTRAN composition | `Table` (step, state, basin) |
| `leech_paxos_full` | Leech lattice consensus | `KeyValue` (round, decision, vector) |
| `ganja_export` | GA visualization data | `MapEntity` per blade/vector |
| `monster_demo` / `monster_tour` | 194 conjugacy classes | `Table` + `Heading` per class |
| `exp*` experiments | numeric results | `Table` or `KeyValue` per experiment |
| `meme_gallery` | meme evolution | `List` of generation summaries |
| `algebra_proofs_demo` | proof output | `Code { language: "lean4" }` |

## Pattern

Every binary that produces structured output:

```rust
use erdfa_publish::*;

fn main() {
    let mut shards: Vec<Shard> = Vec::new();
    let mut manifest = ShardSet::new("mmgroup-{bin_name}");

    // ... existing computation ...

    // Where you currently println!, also emit a shard:
    let s = Shard::new("basin-table", Component::Table {
        headers: vec!["Basin".into(), "AZ Class".into(), "Count".into()],
        rows: basins.iter().map(|(b, c, n)| vec![b.clone(), c.clone(), n.to_string()]).collect(),
    }).with_tags(vec!["basin".into(), "cl15".into()]);
    manifest.add(&s);
    shards.push(s);

    // Export
    let out = format!("{}.tar", env!("CARGO_BIN_NAME"));
    let mut tar = std::fs::File::create(&out).unwrap();
    manifest.to_tar(&shards, &mut tar).unwrap();
    eprintln!("{} shards → {}", shards.len(), out);
}
```

## TUI → Shards

The `ratatui` TUI bins (`basin_gui`, `monster_tui`) render to terminal. To also emit shards, capture the data model before rendering:

```rust
// Before ratatui draw loop:
let screen = Component::Group {
    role: "screen".into(),
    children: vec![
        Component::Heading { level: 1, text: title.clone() },
        Component::Table { headers, rows },
        Component::KeyValue { pairs: status_pairs },
    ],
};
let s = Shard::new("tui-snapshot", screen);
```

This lets the same data render as TUI locally and as CBOR shards for remote/a11y consumers.

## DASL chain → Tree shards

The `dasl_chain` / `dasl_walk` bins produce step-by-step traces. Map to recursive `Tree`:

```rust
let tree = Component::Tree {
    label: "DASL chain".into(),
    children: steps.iter().map(|s| Component::KeyValue {
        pairs: vec![
            ("step".into(), s.index.to_string()),
            ("state".into(), format!("{:?}", s.blade)),
            ("basin".into(), s.basin.to_string()),
        ],
    }).collect(),
};
```

## Existing CBOR compatibility

mmgroup-rust already uses `ciborium` for DASL serialization (`src/dasl.rs`). erdfa-publish uses the same DA51 tag (55889), so existing CBOR consumers will recognize the shards.

## Tags to use

`basin`, `cl15`, `fractran`, `dasl`, `leech`, `monster`, `proof`, `alife`, `moonshine`, `experiment`

## Output convention

Each binary writes `{bin_name}.tar` containing its shards + manifest. The pipeline can then:

```bash
for tar in *.tar; do ipfs add -r "$tar"; done
```

Or feed into RabbitMQ via `kantpaste route`.

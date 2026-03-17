# SOP: Build Shard Indexes

*   **ID:** SOP-ERDFA-006
*   **Author:** Kiro
*   **Status:** Active
*   **Date:** 2026-03-19

## 1. Purpose

To build multi-dimensional indexes over a directory of DA51 CBOR shards, enabling fast lookup by hash, tag, word, name, directory, and git key.

## 2. Scope

Covers the `erdfa-cli index` subcommand. Applies to any shard directory produced by SOP-ERDFA-003 or SOP-ERDFA-005.

## 3. Prerequisites

*   `erdfa-cli` binary built per SOP-ERDFA-001
*   A populated shard directory (e.g. `./shards/kiro-chats/`)

## 4. Procedure

### 4.1. Run Index

```bash
erdfa-cli index --dir ./shards/kiro-chats/
```

Optional: specify a custom output directory:

```bash
erdfa-cli index --dir ./shards/kiro-chats/ --out ./indexes/
```

Default output is `<dir>/indexes/`.

### 4.2. Output Files

The command produces 6 JSON index files:

| File | Key | Value | Description |
|---|---|---|---|
| `hash_index.json` | CID (bafk...) | shard ID | Content-addressed lookup |
| `tag_index.json` | tag string | [shard IDs] | Tag-based filtering |
| `word_index.json` | word (≥3 chars) | [shard IDs] | Inverted text index |
| `name_index.json` | shard ID | filename | Shard ID → file mapping |
| `dir_index.json` | directory path | [shard IDs] | Working directory hierarchy |
| `git_index.json` | full path | [shard IDs] | Git repo / project paths |

### 4.3. Verify

Check the JSON summary output:

```json
{"shards_indexed":161178,"tags":7,"hashes":161178,"names":161178,"directories":64,"words":85396,"git_keys":76,"output":"./shards/kiro-chats/indexes"}
```

Spot-check an index:

```bash
python3 -c "import json; d=json.load(open('./shards/kiro-chats/indexes/tag_index.json')); print({k:len(v) for k,v in d.items()})"
```

## 5. Index Dimensions

*   **hash** — CID → shard ID. Every shard has a unique SHA-256 content hash (bafk-prefixed).
*   **tag** — tag → [shard IDs]. Tags include `kiro`, `chat`, `meta`, `cft`, `cft.post`, `cft.paragraph`, `arrow`.
*   **word** — lowercase word → [shard IDs]. Extracted from Paragraph content and KeyValue pairs. Words shorter than 3 characters are excluded.
*   **name** — shard ID → filename. Reverse lookup from logical ID to filesystem path.
*   **directory** — directory path → [shard IDs]. Derived from the `key` field in meta shards, expanded to all ancestor directories.
*   **git** — full working directory path → [shard IDs]. The raw `key` value from meta shards (typically a git repo working directory).

## 6. Troubleshooting

*   If `directories` and `git_keys` are 0, the meta shards may have empty `key` fields. Re-run `erdfa-cli parquet` to regenerate (the key is read from the parquet `key` column).
*   Large shard directories (>100K files) may take several seconds to index.

## 7. Related Documents

*   SOP-ERDFA-001: Build and Develop
*   SOP-ERDFA-003: Export Kiro Chat History
*   SOP-ERDFA-005: Incremental Chat Refresh

# SOP: Incremental Chat Refresh

*   **ID:** SOP-ERDFA-005
*   **Author:** Kiro
*   **Status:** Active
*   **Date:** 2026-03-19

## 1. Purpose

To incrementally import new Kiro chat conversations from parquet exports without reprocessing existing shards.

## 2. Scope

Covers the `erdfa-cli refresh` subcommand. Applies when new conversations have been added to the parquet export since the last full or incremental import.

## 3. Prerequisites

*   `erdfa-cli` binary built per SOP-ERDFA-001
*   Existing shard directory from a prior SOP-ERDFA-003 run
*   Updated parquet export containing new conversations

## 4. Procedure

### 4.1. Run Refresh

```bash
erdfa-cli refresh \
    --src /path/to/parquet-export/ \
    --dir ./shards/kiro-chats/ \
    --max-depth 1
```

Or via Makefile:

```bash
make refresh SRC=/path/to/parquet-export/ DIR=./shards/kiro-chats/ DEPTH=1
```

The command:
1.  Scans `*_meta.cbor` files in the output directory to collect already-processed conversation IDs.
2.  Reads all `conversations_v2_chunk_*.parquet` files from the source directory.
3.  Skips conversations whose ID already exists in the shard directory.
4.  Generates meta, CFT, and arrow shards for new conversations only.

### 4.2. Verify

Check the JSON summary output:

```json
{"existing":861,"skipped":866,"new_conversations":5,"new_shards":935}
```

*   `existing` — conversations found in the shard directory
*   `skipped` — conversations in parquet that were already processed
*   `new_conversations` — newly imported conversations
*   `new_shards` — total new shard files written

### 4.3. Re-index

After refresh, rebuild indexes per SOP-ERDFA-006:

```bash
erdfa-cli index --dir ./shards/kiro-chats/
```

## 5. Troubleshooting

*   If `existing` count is lower than expected, some conversations may share 8-character UUID prefixes (short ID collisions). The full conversation_id is stored in the meta shard.
*   If `new_conversations` is 0 but you expected new data, verify the parquet export was regenerated from the latest database.

## 6. Related Documents

*   SOP-ERDFA-001: Build and Develop
*   SOP-ERDFA-003: Export Kiro Chat History
*   SOP-ERDFA-006: Build Shard Indexes

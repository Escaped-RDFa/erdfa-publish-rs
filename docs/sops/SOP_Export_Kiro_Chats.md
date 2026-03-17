# SOP: Export Kiro Chat History to CBOR Shards

*   **ID:** SOP-ERDFA-003
*   **Author:** Kiro
*   **Status:** Active
*   **Date:** 2026-03-19

## 1. Purpose

To provide a repeatable process for converting Kiro CLI chat history (stored as parquet exports) into DA51-tagged CBOR shards suitable for semantic indexing, IPFS publishing, and eRDFa rendering.

## 2. Scope

This SOP covers the end-to-end pipeline from Kiro SQLite database → parquet export → CBOR shard generation. It applies to the conversation data in `conversations_v2` table.

## 3. Prerequisites

*   Kiro CLI database at `~/.local/share/kiro-cli/data.sqlite3`
*   Python 3 with `pandas` and `pyarrow` installed (for Step 1 only)
*   `erdfa-cli` binary built per SOP-ERDFA-001 (includes parquet support)
*   The parquet export script (`export_chats.py`) or a fresh export

## 4. Input Data Schema

The `conversations_v2` parquet files contain:

| Column | Type | Description |
|---|---|---|
| `key` | string | Working directory / context path |
| `conversation_id` | string | UUID |
| `value` | string | JSON blob containing full conversation |
| `created_at` | int64 | Unix timestamp (ms) |
| `updated_at` | int64 | Unix timestamp (ms) |

The `value` JSON contains:
*   `conversation_id` — UUID
*   `history[]` — array of turns, each with `user` and `assistant` objects
*   `latest_summary` — conversation summary text
*   `model_info` — model used

Each turn's `user` object has `content` (dict with message text) and `timestamp`.
Each turn's `assistant` object is either `{"Message": ...}` or `{"ToolUse": {"content": ..., "tool_uses": [...]}}`.

## 5. Procedure

### 5.1. Step 1: Export Chat Database to Parquet

*   **Objective:** Extract conversations from SQLite to portable parquet files.
*   **Action:**
    1.  Create an export directory:
        ```bash
        export EXPORT_DIR=./kiro-chat-backup
        mkdir -p "$EXPORT_DIR"
        ```
    2.  Run the export script:
        ```bash
        python3 ./export_chats.py
        ```
        Or re-export from the database directly (see export_chats.py for reference).
    3.  Verify chunk files exist:
        ```bash
        ls -lh "$EXPORT_DIR"/conversations_v2_chunk_*.parquet
        ```

### 5.2. Step 2: Import Parquet as CBOR Shards

*   **Objective:** Convert parquet chat exports directly to DA51 CBOR shards using `erdfa-cli parquet`.
*   **Action:**
    ```bash
    erdfa-cli parquet \
        --src "$EXPORT_DIR" \
        --dir ./shards/kiro-chats/ \
        --max-depth 1
    ```
    Or via Makefile:
    ```bash
    make parquet SRC="$EXPORT_DIR" DIR=./shards/kiro-chats/ DEPTH=1
    ```
    This reads all `conversations_v2_chunk_*.parquet` files, parses the JSON conversation data, and emits:
    1.  One `<short_id>_meta.cbor` metadata shard per conversation (conversation_id, created_at, key, turns).
    2.  CFT-decomposed shards from user messages, assistant responses, and conversation summaries.
    3.  Arrow shards linking CFT layers.

### 5.3. Step 3: Verify Output

*   **Objective:** Confirm shards were generated correctly.
*   **Action:**
    1.  Check the JSON summary output from the command:
        ```json
        {"parquet_files":9,"conversations":866,"shards":161922,"dir":"./shards/kiro-chats/"}
        ```
    2.  List and spot-check:
        ```bash
        erdfa-cli list ./shards/kiro-chats/ | python3 -m json.tool | head -20
        ```
    3.  Show a specific shard:
        ```bash
        erdfa-cli show ./shards/kiro-chats/<short_id>_meta.cbor
        erdfa-cli show ./shards/kiro-chats/<short_id>_post.cbor
        ```

## 6. Output Specification

Each conversation produces:
*   One `<conversation_id>_post.cbor` — whole conversation shard
*   Paragraph-level shards for each turn
*   Line-level shards (at depth ≥ 2)
*   Arrow shards linking layers

Tags: `kiro`, `chat`, `cft.*` scale tags, conversation_id

## 7. Automation

For recurring exports, add a cron entry or Makefile target:
```makefile
export-chats:
	python3 scripts/parquet_to_text.py --src $(PARQUET_DIR) --out /tmp/kiro-text/
	$(MAKE) import SRC=/tmp/kiro-text/ DIR=./shards/kiro-chats/ DEPTH=2
```

## 8. Related Documents

*   SOP-ERDFA-001: Build and Develop erdfa-publish
*   SOP-ERDFA-002: Import Files as CBOR Shards
*   `export_chats.py` in the project directory
*   Kiro DB: `~/.local/share/kiro-cli/data.sqlite3`

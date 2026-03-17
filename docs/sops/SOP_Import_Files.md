# SOP: Import Files as CBOR Shards

*   **ID:** SOP-ERDFA-002
*   **Author:** Kiro
*   **Status:** Active
*   **Date:** 2026-03-19

## 1. Purpose

To provide a repeatable process for importing a directory of text-based files into DA51-tagged CBOR shards with full Conformal Field Tower (CFT) decomposition.

## 2. Scope

This SOP covers the `erdfa-cli import` subcommand. It applies to any directory of supported file types: `.txt`, `.md`, `.org`, `.rs`, `.el`, `.json`, `.toml`, `.yaml`, `.yml`, `.sh`, `.py`.

## 3. Prerequisites

*   `erdfa-cli` binary built per SOP-ERDFA-001
*   A source directory containing text-based files
*   An output directory (will be created if absent)

## 4. Procedure

### 4.1. Identify Source and Output Directories

*   **Objective:** Determine the input files and where shards will be written.
*   **Action:**
    1.  Identify the source directory containing files to import.
    2.  Choose or create an output directory for CBOR shards.

### 4.2. Choose CFT Decomposition Depth

*   **Objective:** Select the granularity of the conformal tower decomposition.
*   **Reference:**

    | Depth | Layers included | Typical use |
    |-------|----------------|-------------|
    | 0 | Post only | Whole-document shards |
    | 1 | + Paragraph | Section-level |
    | 2 (default) | + Line | Line-level granularity |
    | 3 | + Token | Word-level analysis |
    | 4 | + Emoji | Emoji extraction |
    | 5 | + Byte | Full decomposition |

### 4.3. Execute the Import

*   **Objective:** Run the import and produce CBOR shards.
*   **Action:**
    ```bash
    make import SRC=<source-dir> DIR=<output-dir> DEPTH=<0-5>
    ```
    Or directly:
    ```bash
    result/bin/erdfa-cli import --src <source-dir> --dir <output-dir> --max-depth <0-5>
    ```

*   **Example:**
    ```bash
    make import SRC=./examples DIR=./shards/examples DEPTH=2
    ```

### 4.4. Verify Output

*   **Objective:** Confirm shards were written correctly.
*   **Action:**
    1.  List the generated shards:
        ```bash
        make list DIR=<output-dir>
        ```
    2.  Inspect an individual shard:
        ```bash
        result/bin/erdfa-cli show <output-dir>/<shard-id>.cbor
        ```
    3.  Verify the JSON output contains `id`, `cid` (bafk-prefixed), `tags` (including `cft.*` scale tags), and `component`.

*   **Expected output format:**
    ```json
    [{"file":"example_post.cbor","id":"example_post","cid":"bafk...","tags":["cft","cft.post"],"type":"key_value"}]
    ```

## 5. Output Structure

Each imported file produces:
*   **Shards:** One per CFT layer node (post, paragraph, line, token, etc.)
*   **Arrows:** One per parent→child relationship between layers
*   **Naming:** `<stem>_post`, `<stem>_p0`, `<stem>_p0_l0`, `<stem>_p0_l0_t0`, etc.
*   **Format:** DA51-tagged CBOR (tag 55889), content-addressed via SHA-256

## 6. Troubleshooting

| Symptom | Cause | Resolution |
|---|---|---|
| `cannot read src dir` | Source directory doesn't exist | Verify path with `ls` |
| File skipped silently | Unsupported extension or binary file | Only supported text extensions are processed |
| 0 shards produced | All files are binary or empty | Check file contents and extensions |

## 7. Related Documents

*   SOP-ERDFA-001: Build and Develop erdfa-publish
*   `src/cft.rs` — CFT decomposition implementation
*   `src/bin/erdfa-cli.rs` — CLI `import` subcommand

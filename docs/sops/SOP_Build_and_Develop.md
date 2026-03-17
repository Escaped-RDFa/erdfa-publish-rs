# SOP: Build and Develop erdfa-publish

*   **ID:** SOP-ERDFA-001
*   **Author:** Kiro
*   **Status:** Active
*   **Date:** 2026-03-19

## 1. Purpose

To provide a repeatable process for building the `erdfa-publish` Rust crate and entering a development environment using the Nix flake and Makefile.

## 2. Scope

This SOP applies to anyone building, developing, or testing the `erdfa-publish` crate on a system with Nix flakes enabled.

## 3. Prerequisites

*   Nix with flakes enabled (`experimental-features = nix-command flakes` in `nix.conf`)
*   Git (all source files must be tracked — Nix ignores untracked files)
*   The `erdfa-publish` repository cloned locally

## 4. Procedure

### 4.1. Build the Binary

*   **Objective:** Produce the `erdfa-cli` binary via Nix.
*   **Action:**
    1.  Navigate to the repository root.
    2.  Run:
        ```bash
        make build
        ```
    3.  The binary is produced at `result/bin/erdfa-cli`.

*   **Verification:**
    ```bash
    result/bin/erdfa-cli --help
    ```
    Expected: help text listing `list`, `show`, `create`, `import` subcommands.

### 4.2. Enter the Development Shell

*   **Objective:** Get a shell with `cargo`, `rustc`, `rust-analyzer`, `rustfmt`, and `clippy`.
*   **Action:**
    ```bash
    make develop
    ```
*   **Verification:**
    ```bash
    cargo --version && rustc --version
    ```

### 4.3. Run Checks and Tests

*   **Objective:** Verify the crate compiles and all tests pass.
*   **Action:**
    ```bash
    make check   # cargo check inside nix dev shell
    make test    # cargo test inside nix dev shell
    ```
*   **Verification:** Both commands exit with status 0 and no errors.

### 4.4. Clean Build Artifacts

*   **Objective:** Remove all build artifacts for a fresh build.
*   **Action:**
    ```bash
    make clean
    ```
    This removes `target/` (via `cargo clean`) and the `result` symlink.

## 5. Troubleshooting

| Symptom | Cause | Resolution |
|---|---|---|
| `Path 'Cargo.lock' is not tracked by Git` | Untracked files invisible to Nix | `git add` the file, then rebuild |
| `error: experimental Nix feature 'flakes' is disabled` | Flakes not enabled | Add `experimental-features = nix-command flakes` to `/etc/nix/nix.conf` |
| Stale build after code changes | Nix caches by git tree hash | `git add -A && git commit` then `make build` |

## 6. Related Files

*   `flake.nix` — Nix flake definition (packages + devShell)
*   `Makefile` — Make targets wrapping Nix commands
*   `Cargo.toml` — Rust crate manifest
*   `Cargo.lock` — Pinned dependency versions (must be git-tracked)

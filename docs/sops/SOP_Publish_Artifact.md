# SOP: Publish erdfa-publish Artifact to Git

*   **ID:** SOP-ERDFA-004
*   **Author:** Kiro
*   **Status:** Active
*   **Date:** 2026-03-19
*   **Related:** CRQ-032 (Dynamic Nix Artifact Publishing)

## 1. Purpose

To provide a repeatable process for building the `erdfa-publish` Nix package and publishing the resulting NAR archive to the `crq-binstore` Git repository.

## 2. Scope

This SOP covers building the flake output, exporting to NAR, and pushing to the binary store. It follows the dynamic publishing pattern established in SOP-009 (ai-ml-zk-ops).

## 3. Prerequisites

*   Nix with flakes enabled
*   `erdfa-publish` repository with all files git-tracked (per SOP-ERDFA-001)
*   Write access to `https://github.com/meta-introspector/crq-binstore.git`
*   The `publish_nix_artifact_to_git.sh` script from ai-ml-zk-ops, or equivalent

## 4. Procedure

### 4.1. Step 1: Build the Flake

*   **Objective:** Produce the Nix store path for the erdfa-publish package.
*   **Action:**
    ```bash
    cd /path/to/erdfa-publish
    make build
    ```
*   **Verification:**
    ```bash
    readlink result
    # Expected: /nix/store/<hash>-erdfa-publish-0.1.0
    ```

### 4.2. Step 2: Export to NAR

*   **Objective:** Create a portable Nix Archive from the store path.
*   **Action:**
    ```bash
    STORE_PATH=$(readlink result)
    nix-store --export "$STORE_PATH" > erdfa-publish.nar
    ```
*   **Verification:**
    ```bash
    ls -lh erdfa-publish.nar
    # Expected: non-zero file size
    ```

### 4.3. Step 3: Push to crq-binstore

*   **Objective:** Commit and push the NAR to the binary store repository.
*   **Action:**
    Using the ai-ml-zk-ops publishing script:
    ```bash
    /path/to/scripts/publish_nix_artifact_to_git.sh \
        ".#packages.x86_64-linux.erdfa-publish" \
        "https://github.com/meta-introspector/crq-binstore.git"
    ```
    Or manually:
    ```bash
    cd /path/to/crq-binstore
    git pull origin main
    cp /path/to/erdfa-publish/erdfa-publish.nar .
    git add erdfa-publish.nar
    git commit -m "publish: erdfa-publish $(date -I)"
    git push origin main
    ```

### 4.4. Step 4: Verify Publication

*   **Objective:** Confirm the artifact is available in the binary store.
*   **Action:**
    1.  Check the crq-binstore repository for the new commit.
    2.  Verify the NAR file is present and non-empty.
    3.  Optionally import on another machine:
        ```bash
        nix-store --import < erdfa-publish.nar
        ```

## 5. Push Source to GitHub

*   **Objective:** Keep the source repository up to date.
*   **Action:**
    ```bash
    cd /path/to/erdfa-publish
    git push origin main
    ```
*   **Remote:** `https://github.com/Escaped-RDFa/erdfa-publish-rs.git`

## 6. Related Documents

*   SOP-ERDFA-001: Build and Develop erdfa-publish
*   SOP-009: Dynamic Nix Artifact Publishing (ai-ml-zk-ops)
*   CRQ-032: Dynamic Nix Artifact Publishing
*   `flake.nix` — package definition

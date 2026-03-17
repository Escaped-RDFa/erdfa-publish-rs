# erdfa-publish — Standard Operating Procedures

## Index

| ID | Title | Status | Description |
|---|---|---|---|
| SOP-ERDFA-001 | [Build and Develop](sops/SOP_Build_and_Develop.md) | Active | Build `erdfa-cli` binary and enter dev shell via Nix flake + Makefile |
| SOP-ERDFA-002 | [Import Files as CBOR Shards](sops/SOP_Import_Files.md) | Active | Import text files with CFT decomposition into DA51 CBOR shards |
| SOP-ERDFA-003 | [Export Kiro Chat History](sops/SOP_Export_Kiro_Chats.md) | Active | Pipeline: Kiro SQLite → parquet → DA51 CBOR shards (via `erdfa-cli parquet`) |
| SOP-ERDFA-004 | [Publish Artifact to Git](sops/SOP_Publish_Artifact.md) | Active | Build NAR and push to crq-binstore |
| SOP-ERDFA-005 | [Incremental Chat Refresh](sops/SOP_Refresh_Chats.md) | Active | Import only new conversations from parquet (via `erdfa-cli refresh`) |
| SOP-ERDFA-006 | [Build Shard Indexes](sops/SOP_Build_Indexes.md) | Active | Build hash/tag/word/name/directory/git indexes from shard directory |

## Dependency Graph

```
SOP-ERDFA-001 (Build)
    ├── SOP-ERDFA-002 (Import) ── requires built binary
    ├── SOP-ERDFA-003 (Chat Export) ── requires SOP-002 + parquet data
    │   ├── SOP-ERDFA-005 (Refresh) ── requires prior SOP-003 run
    │   └── SOP-ERDFA-006 (Index) ── requires shard directory from SOP-003 or SOP-005
    └── SOP-ERDFA-004 (Publish) ── requires successful build
```

## Conventions

*   All SOPs follow ISO 9000 structure: Purpose, Scope, Prerequisites, Procedure, Verification, Troubleshooting, Related Documents.
*   SOP IDs use the `SOP-ERDFA-NNN` prefix.
*   Status values: `Draft`, `Active`, `Deprecated`.
*   All procedures must be executable as-is by a new contributor with the listed prerequisites.

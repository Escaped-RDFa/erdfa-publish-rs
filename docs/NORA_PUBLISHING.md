# Publishing erdfa-publish to Nora Registry

This document describes how to configure and publish the `erdfa-publish` package to the Nora artifact registry.

## Overview

[Nora](https://github.com/getnora-io/nora) is a multi-protocol artifact registry that serves as a local cache/proxy for:
- Cargo (Rust packages)
- Go modules
- npm packages
- PyPI packages
- Raw artifacts

The `erdfa-publish` package is configured to publish to a local Nora instance running at `http://127.0.0.1:4000/cargo/index`.

## Prerequisites

1. **Nora Registry Instance**: A running Nora instance with Cargo registry enabled
   ```bash
   # Check if Nora is running
   curl http://127.0.0.1:4000/health
   
   # Should return: {"status":"ok"}
   ```

2. **Nix Flakes**: Ensure Nix flakes are enabled
   ```bash
   # Enable flakes (if not already enabled)
   mkdir -p ~/.config/nix
   echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
   ```

3. **Cargo**: Rust's package manager

## Configuration

### 1. Cargo Registry Configuration

The project uses a `.cargo/config.toml` file to configure Cargo to use Nora as its registry source. This is defined in the Nix flake at `config/cargo-config.nix`.

```toml
[source.crates-io]
replace-with = "nora"

[source.nora]
registry = "http://127.0.0.1:4000/cargo/index"
```

### 2. Flake Configuration

The `flake.nix` file integrates with the crane Nix library and includes Nora-aware configuration:

```nix
{
  inputs = {
    # Standard Nix inputs
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    
    # crane for Rust builds
    crane.url = "github:ipetkov/crane";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  
  outputs = { self, nixpkgs, flake-utils, crane, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; overlays = [ rust-overlay.overlays.default ]; };
        rust = pkgs.rust-bin.stable.latest.default;
        craneLib = (crane.mkLib pkgs).overrideToolchain rust;
        
        # Nora Cargo registry configuration
        noraCargoConfig = pkgs.writeTextDir ".cargo/config.toml" ''
[source.crates-io]
replace-with = "nora"

[source.nora]
registry = "http://127.0.0.1:4000/cargo/index"
'';
      in {
        packages.erdfa-publish = craneLib.buildPackage {
          # ... build configuration ...
        };
      }
    );
}
```

## Publishing Workflow

### Step 1: Build with Nora Configuration

To build erdfa-publish against the local Nora registry:

```bash
# Enter the development shell (auto-configures Nora registry)
nix develop

# Or build directly with Nix
nix build .#erdfa-publish
```

### Step 2: Publish to Nora Registry

Once built, you can publish to the Nora registry:

```bash
# Package and publish using cargo
cargo package

# Upload to Nora (requires running Nora instance)
# Method 1: Using the published package
cargo publish --registry nora

# Method 2: Using the package directory
export NORA_REGISTRY=http://127.0.0.1:4000/cargo/index
cargo package && cargo publish --registry nora
```

### Step 3: Verify Publication

Check that the package was published successfully:

```bash
# List packages in Nora
curl http://127.0.0.1:4000/cargo/index/config.json | jq '.packages[] | select(.name == "erdfa-publish")'

# Or check via the web UI
# Open http://127.0.0.1:4000/cargo/index in your browser
```

### Step 4: Configure .cargo/config.toml for Downstream Users

For projects that depend on `erdfa-publish`, add this to their `.cargo/config.toml`:

```toml
[source.crates-io]
replace-with = "nora"

[source.nora]
registry = "http://127.0.0.1:4000/cargo/index"
```

## Troubleshooting

### Issue: Cannot connect to Nora registry

**Symptom**: `cargo publish` fails with connection error

**Solution**:
```bash
# Verify Nora is running
curl http://127.0.0.1:4000/health

# If not running, start Nora
sudo systemctl start nora.service

# Or run manually from the nora project
cd ~/projects/nora
bash deploy.sh deploy
```

### Issue: Package already published

**Symptom**: `cargo publish` fails with "package already exists" error

**Solution**:
```bash
# Increment the version in Cargo.toml
# Change from:
# version = "0.1.0"
# To:
# version = "0.1.1"

# Update Cargo.lock
cargo update

# Publish again
cargo publish --registry nora
```

### Issue: Dependency resolution fails

**Symptom**: Build fails because dependencies cannot be fetched

**Solution**: Ensure the dependency is available in Nora or configure fallback:

```nix
# In flake.nix, ensure dependencies are vendored or available
buildPackage {
  # ...
  # If a dependency needs to be fetched from crates.io
  cargoVendorDir = ./vendor;
}
```

## Integration with Nora Skills

This project integrates with the following Nora-related skills from `~/projects/dotagents/skills/`:

### crane Skill

The `crane` skill provides patterns for building Rust packages with Nix. See:
- `/home/mdupont/projects/dotagents/skills/crane/SKILL.md`

Key patterns:
- Using `omaster` refs for all flake inputs
- Vendor dependencies via `craneLib.buildPackage`
- Avoiding circular bootstrap (nora must not use nora to build nora)

### nora-car-shmem Skill

The `nora-car-shmem` skill configures Nora to use CAR shared memory storage backend. See:
- `/home/mdupont/projects/dotagents/skills/nora-car-shmem/SKILL.md`

### nora-monitor-tile Skill

The `nora-monitor-tile` skill provides health monitoring for the Nora registry. See:
- `/home/mdupont/projects/dotagents/skills/nora-monitor-tile/SKILL.md`

## Related Documentation

- [Nora README](https://github.com/getnora-io/nora/README.md) - Main Nora documentation
- [Nora Flake Example](https://github.com/getnora-io/nora/flake.nix) - Complete working example
- [Crane Documentation](https://github.com/ipetkov/crane) - Nix Rust build library
- [System Manager](https://github.com/numtide/system-manager) - Service deployment

## Quick Commands Reference

```bash
# Check Nora health
curl http://127.0.0.1:4000/health

# Build erdfa-publish with Nix
nix build .#erdfa-publish

# Enter dev shell (auto-configures Nora registry)
nix develop

# Package for publishing
cargo package

# Publish to Nora
cargo publish --registry nora

# Update flake lock
nix flake lock --update-input <input-name>

# Deploy to system-manager
sudo system-manager switch --flake ~/projects/erdfa-publish#erdfa-publish
```

## License

MIT OR Apache-2.0

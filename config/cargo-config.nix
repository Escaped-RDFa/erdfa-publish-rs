{ pkgs ? import <nixpkgs> {} }:

pkgs.writeTextDir ".cargo/config.toml" ''
[source.crates-io]
replace-with = "nora"

[source.nora]
registry = "http://127.0.0.1:4000/cargo/index"
''

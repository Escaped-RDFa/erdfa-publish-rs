.PHONY: build develop check test clean run list show import parquet refresh index

build:
	nix build

develop:
	nix develop

check:
	nix develop -c cargo check

test:
	nix develop -c cargo test

clean:
	nix develop -c cargo clean
	rm -rf result

run:
	nix run

# CLI shortcuts — usage: make list DIR=./shards
list:
	nix run -- list $(DIR)

show:
	nix run -- show $(FILE)

import:
	nix run -- import --src $(SRC) --dir $(DIR) --max-depth $(or $(DEPTH),2)

parquet:
	nix run -- parquet --src $(SRC) --dir $(DIR) --max-depth $(or $(DEPTH),1)

refresh:
	nix run -- refresh --src $(SRC) --dir $(DIR) --max-depth $(or $(DEPTH),1)

index:
	nix run -- index --dir $(DIR) $(if $(OUT),--out $(OUT))

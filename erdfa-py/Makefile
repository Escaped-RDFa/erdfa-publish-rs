.PHONY: build test clean deploy

build:
	nix develop --command bash -c 'maturin build --release'

develop:
	nix develop --command bash -c 'maturin develop --release'

test: develop
	nix develop --command bash -c 'python3 -c "import erdfa_py; print(dir(erdfa_py))"'
	nix develop --command bash -c 'python3 -c "from zkperf import zkperf, zkperf_dump; print(\"zkperf ready\")"'

test-regs: develop
	nix develop --command python3 -c "import erdfa_py; r=erdfa_py.capture_regs(); print('regs:', {k:hex(v) for k,v in list(r.items())[:4]}); ssp=erdfa_py.regs_to_ssp(r); print('ssp:', list(ssp)); orb=erdfa_py.ssp_to_orbifold(ssp); print('orbifold:', orb)"

clean:
	rm -rf target/ .dev-venv/

deploy: build
	@echo "Wheel: $$(ls target/wheels/*.whl | tail -1)"

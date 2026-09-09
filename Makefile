# Bibliographies. Two modules out of one crate: the parser alone, and the
# markdown renderer with citations set into it.
#
# Built for wasm32-unknown-unknown into dist/, under the names a host loads
# them by. `rustup target add wasm32-unknown-unknown` once.

OUT     := target/wasm32-unknown-unknown/release/wasm_bibliography.wasm
VERSION := $(shell grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
SOURCES := $(shell find src Cargo.toml -type f 2>/dev/null) \
           $(shell find ../wasm-helpers/src ../wasm-markdown/src -type f 2>/dev/null)
KEYS    ?= .keys.yaml

.DEFAULT_GOAL := help
.PHONY: help build compress checksums release secrets test fmt clean

help:  ## Display this help screen
	@printf "\033[1mAvailable commands:\033[0m\n\n"
	@grep -hE '^[a-z.A-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-12s\033[0m %s\n", $$1, $$2}' | sort

build: dist/bibliography.wasm dist/citations.wasm  ## Build both modules

# The parser: no renderer, no comrak, no CSL. A few hundred kilobytes, fetched
# by an editor that wants to complete a citation key.
dist/bibliography.wasm: $(SOURCES)
	@cargo build --release --target wasm32-unknown-unknown --no-default-features --features exports
	@mkdir -p dist
	@cp $(OUT) $@
	@ls -lh $@ | awk '{print "bibliography.wasm", $$5}'

# The renderer with citations set into it, which is markdown plus the whole of
# CSL: seven times the plain renderer, and fetched only for a document that
# cites something.
dist/citations.wasm: $(SOURCES)
	@cargo build --release --target wasm32-unknown-unknown
	@mkdir -p dist
	@cp $(OUT) $@
	@ls -lh $@ | awk '{print "citations.wasm", $$5}'

compress: dist/bibliography.wasm.br dist/citations.wasm.br  ## Pre-compress both for serving

dist/%.wasm.br: dist/%.wasm tools/compress.mjs
	@node tools/compress.mjs $<

checksums: dist/SHA256SUMS  ## Write the digests the application pins

dist/SHA256SUMS: dist/bibliography.wasm.br dist/citations.wasm.br
	@cd dist && sha256sum *.wasm *.wasm.br *.wasm.gz > SHA256SUMS
	@cat $@

release: checksums  ## Publish the version in Cargo.toml as a GitHub release
	@command -v gh >/dev/null || { echo "gh is not installed"; exit 1; }
	@gh auth status >/dev/null 2>&1 || { echo "gh is not signed in: gh auth login"; exit 1; }
	@git diff --quiet || { echo "working tree is dirty; commit before releasing"; exit 1; }
	@gh release view v$(VERSION) >/dev/null 2>&1 \
		&& { echo "v$(VERSION) is already released; bump version in Cargo.toml"; exit 1; } || true
	@gh release create v$(VERSION) dist/* \
		--title "bibliography $(VERSION)" \
		--notes "$$(printf 'Built from %s\n\n```\n%s\n```\n' "$$(git rev-parse --short HEAD)" "$$(cat dist/SHA256SUMS)")"

secrets:  ## Open a shell with the sops-encrypted keys in its environment
	@test -f $(KEYS) || { echo "no $(KEYS) -- see $(KEYS).example"; exit 1; }
	@test -t 0 || { echo "make secrets opens an interactive subshell and needs a terminal" >&2; exit 2; }
	@echo "$(KEYS) is loaded in this shell; exit to drop it"
	@sops exec-env $(KEYS) "$${SHELL:-/bin/sh}"

test:  ## Run the tests natively
	@cargo test

fmt:
	@cargo fmt

clean:
	@rm -rf target dist

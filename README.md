# wasm-bibliography

Bibliographies, and the citations that refer to them. Two WebAssembly modules
out of one crate.

Part of [LibrePaper](https://github.com/LibrePaper).

| Module | What it does | Raw | Brotli |
| --- | --- | ---: | ---: |
| `bibliography.wasm` | parses `.bib` into the library an editor completes from; renders nothing | 282 KiB | **87 KiB** |
| `citations.wasm` | the Markdown renderer with citations set into it | 5.5 MiB | **825 KiB** |

## Why two modules and not one

`citations.wasm` contains the whole of CSL — styles, locales, and the renderer
itself — and is seven times the size of plain
[`wasm-markdown`](https://github.com/LibrePaper/wasm-markdown). A host fetches
the small one to complete a citation key, and the big one only for a document
that actually cites something. Folding them together would make every reader
of a document with no bibliography pay 825 KiB instead of 112 KiB.

Same crate either way: `make build` produces both, one with the `citations`
feature and one without.

## Building

```sh
rustup target add wasm32-unknown-unknown   # once
make build                                 # -> dist/bibliography.wasm, dist/citations.wasm
make compress                              # -> .br and .gz beside them
make test
```

## The interface

The sixteen exports every LibrePaper renderer answers — see
[`wasm-helpers`](https://github.com/LibrePaper/wasm-helpers) — plus one of this
crate's own:

| Export | What it does |
| --- | --- |
| `bibliography(request)` | takes `{main, format, source, texts}` as JSON, returns `{entries, diagnostics}` — the parsed library, with each entry's key, authors, year, title, container, DOI, URL and where it was found |

`bibliography` is not a compile: it leaves no diagnostics behind, rather than
the previous call's.

Built without `citations`, `compile` still exists — the interface is the same
everywhere — and reports *"This module analyzes bibliographies; it does not
render documents."* rather than returning an empty page.

## The `exports` feature

On by default, and **off when another crate links this one as a library**. A
`#[no_mangle]` export belongs to exactly one `cdylib`; two sets of them do not
link. Every LibrePaper renderer carries the same switch for the same reason —
it is what lets `citations.wasm` depend on `wasm-markdown` for real rather than
copying its renderer.

## Licence

MIT. See [LICENSE](LICENSE).

`biblatex`, `hayagriva` and `comrak` carry their own. `hayagriva` bundles the
CSL style and locale archive, which is why `citations.wasm` is the size it is;
those styles are under CC BY-SA, and their terms travel with the artifact.

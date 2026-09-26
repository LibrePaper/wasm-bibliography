//! Bibliographies, and the citations that refer to them.
//!
//! Two modules are built from this one crate, because a host wants them at
//! different moments. `bibliography.wasm` parses `.bib` files so the editor can
//! complete a citation key, and renders nothing at all. `citations.wasm` is the
//! markdown renderer with citation processing on top, and is fetched only for a
//! document that actually cites something -- it is seven times the size of the
//! plain renderer, which is the whole reason the two are not one module.
//!
//! The page template, the diagnostics and the WebAssembly interface come from
//! `wasm-helpers`; the renderer citations are set into comes from
//! `wasm-markdown`, as a real dependency rather than a copy, because citation
//! processing rewrites the same syntax tree comrak produced.

/// The shared page template and the shape a compile answers in.
pub use wasm_helpers::{diagnostic, page};

/// The renderer citations are set into, under the name `citations.rs` expects.
#[cfg(feature = "citations")]
pub use wasm_markdown::markdown;

pub mod bib;

#[cfg(feature = "citations")]
pub mod citations;

#[cfg(all(target_arch = "wasm32", feature = "exports"))]
mod abi;

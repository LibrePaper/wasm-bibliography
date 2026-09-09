//! The seventeen exports a host calls, and nothing else.
//!
//! Sixteen of them are the interface every LibrePaper renderer answers; each
//! wraps the shared implementation in `wasm_helpers::abi`. The seventeenth,
//! `bibliography`, is this crate's own: it hands back a parsed library as JSON
//! and compiles nothing.
//!
//! Which renderer `compile` is depends on how the crate was built. With
//! `citations` it is markdown with its citations set; without, this module
//! parses bibliographies and says so rather than pretending to render.

use wasm_helpers::abi;
use wasm_helpers::diagnostic::Compiled;

/// The library the last compile prepared, kept across prose edits because a
/// worker serialises ABI calls and reparsing every `.bib` on every keystroke is
/// the expensive half of a render. Source-dependent work -- which keys the
/// document actually cites, and what is missing -- is redone every time.
#[cfg(feature = "citations")]
static mut CITATION_LIBRARY: Option<(u64, crate::citations::PreparedLibrary)> = None;

/// Markdown, with citations resolved against whatever `.bib` files the host
/// handed over.
#[cfg(feature = "citations")]
fn render(source: &str, title: &str) -> Compiled {
    let main = match abi::main_name() {
        name if name.is_empty() => "main.md".to_string(),
        name => name,
    };
    let texts: std::collections::BTreeMap<String, String> = abi::files()
        .into_iter()
        .filter_map(|(path, bytes)| String::from_utf8(bytes).ok().map(|text| (path, text)))
        .collect();
    let key = crate::bib::cache_key(&main, "markdown", source, &texts);
    unsafe {
        let cache = &mut *std::ptr::addr_of_mut!(CITATION_LIBRARY);
        if cache.as_ref().is_none_or(|(previous, _)| *previous != key) {
            let library = crate::bib::library(&main, "markdown", source, &texts);
            *cache = Some((key, crate::citations::prepare(&library)));
        }
        crate::citations::compile_prepared(&main, source, title, &cache.as_ref().unwrap().1, &|
            path: &str,
        | abi::asset_url(path))
    }
}

/// Built without the renderer, this module still answers `compile` -- the
/// interface is the same everywhere -- and says what it is instead of
/// returning an empty document.
#[cfg(not(feature = "citations"))]
fn render(_: &str, _: &str) -> Compiled {
    Compiled::failed("This module analyzes bibliographies; it does not render documents.")
}

#[cfg(feature = "citations")]
fn heading(source: &str) -> String {
    crate::markdown::title_of(source)
}

#[cfg(not(feature = "citations"))]
fn heading(_: &str) -> String {
    String::new()
}

/// A parsed bibliography, as the JSON library the editor completes from. Not a
/// compile: it leaves no diagnostics behind, rather than the previous call's.
///
/// # Safety
/// The pointer and length must describe UTF-8 written into this module.
#[no_mangle]
pub unsafe extern "C" fn bibliography(pointer: *const u8, len: usize) -> usize {
    abi::clear_diagnostics();
    abi::answer(crate::bib::analyze_json(abi::text_at(pointer, len)))
}

#[no_mangle]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    abi::alloc(len)
}

/// # Safety
/// `pointer` and `len` must be exactly what a previous `alloc` returned.
#[no_mangle]
pub unsafe extern "C" fn dealloc(pointer: *mut u8, len: usize) {
    abi::dealloc(pointer, len)
}

/// # Safety
/// The pointers and lengths must describe UTF-8 written into this module.
#[no_mangle]
pub unsafe extern "C" fn compile(
    source: *const u8,
    source_len: usize,
    title: *const u8,
    title_len: usize,
) -> usize {
    let source = abi::text_at(source, source_len);
    let title = abi::text_at(title, title_len);
    abi::answer_compiled(render(source, title))
}

/// # Safety
/// `source` and `len` must describe UTF-8 written into this module.
#[no_mangle]
pub unsafe extern "C" fn title_of(source: *const u8, len: usize) -> usize {
    abi::answer(Ok(heading(abi::text_at(source, len))))
}

/// # Safety
/// `title` and `len` must describe UTF-8 written into this module.
#[no_mangle]
pub unsafe extern "C" fn failure_page(title: *const u8, len: usize) -> usize {
    abi::failure_page(title, len)
}

/// # Safety
/// The pointers and lengths must describe UTF-8 written into this module.
#[no_mangle]
pub unsafe extern "C" fn word_diff(
    old: *const u8,
    old_len: usize,
    new: *const u8,
    new_len: usize,
) -> usize {
    abi::word_diff(old, old_len, new, new_len)
}

/// # Safety
/// The pointers and lengths must describe memory written into this module.
#[no_mangle]
pub unsafe extern "C" fn add_file(
    path: *const u8,
    path_len: usize,
    body: *const u8,
    body_len: usize,
) {
    abi::add_file(path, path_len, body, body_len)
}

#[no_mangle]
pub extern "C" fn clear_files() {
    abi::clear_files()
}

/// # Safety
/// The pointers and lengths must describe UTF-8 written into this module.
#[no_mangle]
pub unsafe extern "C" fn set_asset_url(
    path: *const u8,
    path_len: usize,
    url: *const u8,
    url_len: usize,
) {
    abi::set_asset_url(path, path_len, url, url_len)
}

/// # Safety
/// The pointer and length must describe UTF-8 written into this module.
#[no_mangle]
pub unsafe extern "C" fn set_main(path: *const u8, len: usize) {
    abi::set_main(path, len)
}

/// Accepted and ignored: markdown has no clock and no dates.
#[no_mangle]
pub extern "C" fn set_today(year: i32, month: u32, day: u32) {
    abi::set_today(year, month, day)
}

#[no_mangle]
pub extern "C" fn output_ptr() -> *const u8 {
    abi::output_ptr()
}

#[no_mangle]
pub extern "C" fn ok() -> u32 {
    abi::ok()
}

#[no_mangle]
pub extern "C" fn output_kind() -> u32 {
    abi::output_kind()
}

#[no_mangle]
pub extern "C" fn diagnostics() -> usize {
    abi::diagnostics()
}

#[no_mangle]
pub extern "C" fn diagnostics_ptr() -> *const u8 {
    abi::diagnostics_ptr()
}

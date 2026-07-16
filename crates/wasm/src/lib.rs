//! WebAssembly bindings for the Open Construction Modeler engine.
//!
//! This crate is the browser adapter: it exposes the same tested Rust parsing
//! and geometry logic the desktop app uses (via Tauri) to the web app (via
//! WASM), so neither side re-implements it. Built with `wasm-pack build
//! crates/wasm --target web`.

use wasm_bindgen::prelude::*;

/// Parse IFC file contents (STEP text) into the normalized construction
/// objects, returned as a JSON array string.
///
/// The result mirrors exactly what the desktop `load_project` command returns
/// for a single IFC file — each object carries its `render_shape` — so the web
/// viewer can consume it identically to the desktop viewer. Parsing itself is
/// infallible (unresolved geometry falls back rather than erroring); the
/// `Result` only surfaces a JSON serialization failure, which shouldn't happen
/// in practice.
#[wasm_bindgen]
pub fn parse_ifc(contents: &str) -> Result<String, JsError> {
    let mut objects = ifc::parser::parse_ifc_contents(contents);
    for obj in &mut objects {
        obj.render_shape = Some(engine::render::shape_for(obj));
    }
    serde_json::to_string(&objects).map_err(|e| JsError::new(&e.to_string()))
}

/// Parse IFC contents and return the coordinate-drift / alignment report as
/// JSON (see `engine::align`). A standalone check on a single model: flags a
/// model that sits far from the origin (lost base/survey point) and individual
/// objects flung far outside the main cluster (misplaced). Issue #23.
#[wasm_bindgen]
pub fn alignment_report(contents: &str) -> Result<String, JsError> {
    let objects = ifc::parser::parse_ifc_contents(contents);
    let report = engine::align::alignment_report(&objects);
    serde_json::to_string(&report).map_err(|e| JsError::new(&e.to_string()))
}

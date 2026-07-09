# Open Construction Modeler
An open-source, construction-first modeling engine built for VDC coordinators, 
trade contractors, and construction technology teams.

Built because existing tools like Revit and Navisworks were designed for 
architects — not for the people who actually build things.

---

## What This Is

A two-part ecosystem:

**Part 1 — Desktop Modeling Engine** (this repo)  
A fast, construction-centric 3D modeling engine with first-class support for 
trade coordination, assembly modeling, clash detection, and construction sequencing.

**Part 2 — Collaboration Platform** (coming soon)  
A web-based hub for model sharing, VDC knowledge, scripts, and community — 
think GitHub + StackOverflow for construction modeling.

---

## Why Not Just Use Revit?

- Revit loads entire models into memory — crashes on large projects
- Trade coordination requires exporting to Navisworks manually
- Civil/site coordination with DWG is painful
- Workflows designed for architects, not contractors

We're building for the people doing VDC coordination every day.

---

## Current Status

*Last updated: 2026-07-08*

Active development — geometry pipeline, clash detection, and BCF export all
working end-to-end, validated against a real building model (not just
synthetic test fixtures).

**Working:**
- Construction object data model (trade, LOD, CSI code, phase, status)
- Object relationships (dependencies, sequencing, hosting, assemblies)
- Project save/load to `.ocm` format, with round-trip JSON serialization
- IFC 4.x and 2x3 parser with entity indexing
- Full geometry extraction pipeline:
  - `IFCPRODUCTDEFINITIONSHAPE → IFCSHAPEREPRESENTATION → geometry` traversal
  - `IFCEXTRUDEDAREASOLID` depth extraction
  - `IFCTRIANGULATEDFACESET` bounding box extraction
  - `IFCBOUNDINGBOX` fallback for Brep/mapped-representation bodies (doors,
    windows, shared structural members) the extractor can't resolve precisely
  - Unit scale detection (mm, cm, m, ft, in)
  - World matrix computation via IFCLOCALPLACEMENT parent chain
- DXF import for civil/site data (lines, polylines, circles, survey points),
  classified as `Trade::Civil`, with `$INSUNITS` unit-scale detection —
  true binary DWG is a proprietary format with no legal open parser, same
  limitation every other open BIM/CAD tool has
- Clash detection engine: AABB broad-phase with a uniform spatial-hash grid
  so it scales to large models (near-linear on spatially distributed input
  instead of the naive O(n²) all-pairs scan — verified identical to brute
  force by a randomized parity test), severity ranking (Minor/Major/Critical
  by penetration ratio), explicit per-object skip reasons instead of silently
  dropping bad geometry (missing position, missing dimensions, degenerate or
  non-finite geometry)
- BCF 2.1 issue export — clash results export as a `.bcfzip` archive
  readable by Revit, Navisworks, and Procore, with XML-escaped object names
- Tauri 2.0 desktop app with Three.js WebGL viewport
- Trade-color-coded 3D rendering with real IFC/DXF dimensions, red
  highlighting of clashing objects, clash-count badge and clash list panel
- Raycasting object selection (click in viewport → highlights in sidebar)
- Bidirectional selection (sidebar ↔ viewport)
- Native file open dialog (.ifc, .dxf, and .ocm) and save dialog (BCF export)
- Trade filter toggles
- Restrictive Tauri CSP (was previously disabled)
- DoS-hardened file input: every untrusted parser (IFC, DXF, `.ocm`) reads
  through a single bounded reader that caps input size (guarding against a
  malformed/hostile multi-gigabyte file exhausting memory) and rejects path
  traversal
- CI: full workspace test suite (including the Tauri backend, which used to
  be untested), `cargo clippy -D warnings`, `cargo audit`, frontend lint +
  typecheck + build + `npm audit`
- 107 passing Rust unit tests + 19 frontend tests, typecheck/lint clean

**Known limitations:**
- Geometry resolution still falls back to a coarse/placeholder box for
  entity types with no `Body` or `Box` shape representation at all — most
  common real-world exports carry one or the other, but it's not guaranteed
- Clash detection is broad-phase only (AABB overlap) — no narrow-phase
  exact geometry intersection yet, so results can include false positives
  for irregularly-shaped objects whose bounding boxes overlap but whose
  actual geometry doesn't
- DXF import doesn't yet handle `Polyline` (heavy 3D polylines with
  separately-linked vertex entities), blocks/inserts, splines, or text —
  only `Line`, `LwPolyline`, `Circle`, and `Point`

**In Progress / Not Started:**
- Procore integration (needs a developer OAuth app — blocked on credentials)
- 4D schedule integration
- 5D cost integration
- Python scripting layer (PyO3)
- Web collaboration platform

---

## Architecture
```
open-construction-modeler/
├── crates/
│   ├── engine/     — Core data model, relationships, project persistence,
│   │                 clash detection, BCF export
│   ├── ifc/        — IFC parser, geometry extraction, world matrix
│   ├── civil/      — DXF parser for civil/site data
│   └── app/        — CLI application binary
├── frontend/       — React + TypeScript + Three.js viewport
└── src-tauri/      — Tauri 2.0 desktop shell + Rust command bridge
```

**Language:** Rust + TypeScript  
**Desktop Shell:** Tauri 2.0  
**Rendering:** Three.js (WebGL)  
**Scripting:** Python via PyO3 (planned)  

---

## Building
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js (v18+) and Tauri CLI
npm install -g @tauri-apps/cli

# Clone and build
git clone https://github.com/coramb2/open-construction-modeler.git
cd open-construction-modeler

# Run tests (whole workspace, including the Tauri backend)
cargo test --workspace

# Lint
cargo clippy --workspace --all-targets

# Run desktop app (dev mode)
cargo tauri dev
```

---

## Roadmap

- [x] Core data model and project persistence
- [x] IFC 4.x/2x3 import
- [x] Tauri desktop app with Three.js viewport
- [x] IFC geometry extraction pipeline (incl. bounding-box fallback)
- [x] Clash detection engine (AABB broad-phase, severity ranking)
- [x] BCF 2.1 issue export
- [x] DXF civil import (lines, polylines, circles, survey points)
- [ ] Procore integration — blocked on OAuth developer credentials
- [ ] Clash detection narrow-phase (exact geometry intersection)
- [ ] 4D schedule integration
- [ ] 5D cost integration
- [ ] Python scripting layer
- [ ] Lifecycle / change-tracking audit trail
- [ ] Web collaboration platform

See [docs/OCM_Technical_Roadmap_v2.md](docs/OCM_Technical_Roadmap_v2.md) for
the full sprint-by-sprint plan.

---

## Contributing

This project is in active development. If you work in VDC, construction technology, 
or Rust systems programming — contributions and feedback are welcome.

---

## License

Apache-2.0 — see [LICENSE](LICENSE)
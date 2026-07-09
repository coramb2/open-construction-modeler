# Open Construction Modeler
An open-source, construction-first modeling engine built for VDC coordinators, 
trade contractors, and construction technology teams.

Built because existing tools like Revit and Navisworks were designed for 
architects — not for the people who actually build things.

It started as a labor of love and out of plain necessity: the coordination
problems it targets — clashes, lost coordinate systems, the same model drifting
out of alignment as it's handed between trades — are a daily reality on real
jobs, not a hypothetical.

---

## What This Is

A two-part ecosystem:

**Part 1 — Desktop Modeling Engine** (this repo)  
A fast, construction-centric 3D modeling engine with first-class support for 
trade coordination, assembly modeling, clash detection, and construction sequencing.

**Part 2 — Collaboration Platform** (`web/`, active development)  
A web hub that brings the **open-source model** — fork, reuse, and contribute
back — to construction. Not just a place to *host* models, but a place where a
published project, assembly, or made item (a chair, a bracket, anything that
didn't exist before someone built it) can be **forked**, adapted, and its
improvements **proposed back** to the original. It's GitHub's *workflow* applied
to things that get built, not just its storage — see
[Bringing the Open-Source Model to Construction](#bringing-the-open-source-model-to-construction)
below for how fork/diff/merge map onto construction files. Next.js + Supabase,
deployed to Vercel — see [web/SETUP.md](web/SETUP.md).

---

## Why Not Just Use Revit?

- Revit loads entire models into memory — crashes on large projects
- Trade coordination requires exporting to Navisworks manually
- Civil/site coordination with DWG is painful
- Workflows designed for architects, not contractors

We're building for the people doing VDC coordination every day.

---

## Bringing the Open-Source Model to Construction

Software got radically more reusable when GitHub made *forking* the default:
find something close to what you need, copy it with a link back to the original,
change it, and offer your changes upstream. Construction has no equivalent.
Models are locked in proprietary formats, reuse is rare, and the same project
handed to multiple trades drifts out of sync.

Part 2 is aimed squarely at that. Three problems it's built to solve:

**1. Reuse.** Publish a project — or a single assembly or made item — and let
others fork it, build on it, and attribute it back. Lineage is tracked, so you
can see what a design descended from and what's been built from it.

**2. Coordinate & alignment drift.** The most common failure when a project is
federated out to multiple disciplines: the shared coordinate system slips.
Project base point / survey point gets lost, units flip (mm ↔ m ↔ ft), true
north rotates, or the `IFCLOCALPLACEMENT` / `IFCMAPCONVERSION` chain is misread
on re-import — and models that should overlay perfectly end up offset by
hundreds of feet. Because IFC gives every element a stable GUID and a
deterministic placement chain (files carry hard-coded, machine-readable ways to
identify and locate every object), we can resolve each object to world
coordinates and **detect drift directly** — either as a diff before you merge,
or as a standalone "did anything misalign?" check on a single file.

**3. File compatibility.** Revit → AutoCAD → IFC round-trips silently lose
fidelity. The Rust engine in Part 1 already parses IFC/DXF into a normalized,
GUID-anchored object model — the same normalization layer that makes diffing,
alignment-checking, and merging possible in the first place. Leaning on that
engine on the backend (rather than treating uploads as opaque blobs) is the
backbone of a seamless file transition, and the piece most of the remaining
work depends on.

### How fork / diff / merge map onto construction files

Text merges work because git normalizes everything to lines. Construction files
don't diff as text — but normalized, GUID-identified objects do:

- **Fork** — an independent, linked copy of a repo; lineage shown as
  "forked from …".
- **Diff before merge** — a *semantic + spatial* diff: objects added, removed,
  or modified, plus any global coordinate offset / rotation / unit mismatch
  between the two versions.
- **Merge** — reconciled at the object and property level, not the byte level.
  Construction work is already partitioned by discipline (architectural /
  structural / MEP each own different objects), so most merges are additive with
  no conflict; genuine same-object conflicts fall to a human.
- **Validate** — a merged (or freshly uploaded) model is run through the
  existing clash-detection engine and the alignment check, so a merge that is
  data-clean but physically invalid — or simply misaligned — is caught before it
  lands.

> **Status:** the fork / diff / merge / alignment features are the design
> direction for Part 2 and are **not built yet**. The parsing, geometry
> resolution, world-coordinate placement, and clash detection they depend on
> already exist and are tested in Part 1.

---

## Current Status

*Last updated: 2026-07-09*

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

**Web platform (`web/`) — active development:**
- Next.js app (deployed target: Vercel) backed by Supabase — Postgres schema
  (profiles/items/item_images) with Row Level Security on every table, and
  storage buckets for model files + images
- Auth via GitHub OAuth (email/magic-link not added yet)
- Built so far: browse feed, publishing/upload flow, item detail page with an
  in-browser glTF/GLB viewer, and public profile pages
- 31 passing tests (Vitest), including a regression test for an open-redirect
  bypass found and fixed during review (a naive `next.startsWith('/')` check
  doesn't block `//evil.example`, which browsers treat as a protocol-relative
  URL)
- The in-browser viewer supports glTF/GLB only — IFC/DXF parsing lives in Rust
  on the desktop app and hasn't been ported to the browser (WASM) yet; other
  file types are downloadable but not previewable
- Not yet built: fork/lineage, semantic + coordinate diff, merge, and the
  alignment-integrity check — see
  [Bringing the Open-Source Model to Construction](#bringing-the-open-source-model-to-construction)

**In Progress / Not Started:**
- Procore integration (needs a developer OAuth app — blocked on credentials)
- 4D schedule integration
- 5D cost integration
- Python scripting layer (PyO3)

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
├── frontend/       — React + TypeScript + Three.js viewport (desktop app)
├── src-tauri/      — Tauri 2.0 desktop shell + Rust command bridge
└── web/            — Next.js collaboration platform (Part 2), see web/SETUP.md
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
- [x] Web collaboration platform — publish & browse feed, item viewer, profiles
- [ ] Fork & lineage for published models
- [ ] Semantic + coordinate diff (drift / offset / unit-mismatch detection)
- [ ] Alignment-integrity check (standalone + pre-merge)
- [ ] Object/property-level merge, validated by clash detection
- [ ] In-browser IFC/DXF parsing (Rust → WASM)

See [docs/OCM_Technical_Roadmap_v2.md](docs/OCM_Technical_Roadmap_v2.md) for
the full sprint-by-sprint plan.

---

## Contributing

This project is in active development. If you work in VDC, construction technology, 
or Rust systems programming — contributions and feedback are welcome.

---

## License

Apache-2.0 — see [LICENSE](LICENSE)
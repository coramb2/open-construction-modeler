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

Active development — geometry pipeline working end-to-end on real IFC files.

**Working:**
- Construction object data model (trade, LOD, CSI code, phase, status)
- Object relationships (dependencies, sequencing, hosting, assemblies)
- Project save/load to `.ocm` format
- Round-trip JSON serialization
- IFC 4.x and 2x3 parser with entity indexing
- Full geometry extraction pipeline:
  - `IFCPRODUCTDEFINITIONSHAPE → IFCSHAPEREPRESENTATION → geometry` traversal
  - `IFCEXTRUDEDAREASOLID` depth extraction
  - `IFCTRIANGULATEDFACESET` bounding box extraction
  - Unit scale detection (mm, cm, m, ft, in)
  - World matrix computation via IFCLOCALPLACEMENT parent chain
- Tauri 2.0 desktop app with Three.js WebGL viewport
- Trade-color-coded 3D rendering with real IFC dimensions
- Raycasting object selection (click in viewport → highlights in sidebar)
- Bidirectional selection (sidebar ↔ viewport)
- Native file open dialog (.ifc and .ocm)
- Trade filter toggles
- 15 passing unit tests

**In Progress:**
- Y-axis world position (centroid extraction from faceset geometry)
- Extrusion profile dimension extraction (width/depth from IFCARBITRARYCLOSEDPROFILEDEF)
- Clash detection engine

---

## Architecture
```
open-construction-modeler/
├── crates/
│   ├── engine/     — Core data model, relationships, project persistence
│   ├── ifc/        — IFC parser, geometry extraction, world matrix
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

# Run tests
cargo test

# Run desktop app (dev mode)
cargo tauri dev
```

---

## Roadmap

- [x] Core data model and project persistence
- [x] IFC 4.x/2x3 import
- [x] Tauri desktop app with Three.js viewport
- [x] IFC geometry extraction pipeline
- [ ] Y-axis world position from faceset centroids
- [ ] Extrusion profile dimension extraction
- [ ] Clash detection engine
- [ ] BCF 2.1 issue export
- [ ] Procore integration
- [ ] DWG/DXF civil import
- [ ] 4D schedule integration
- [ ] Python scripting layer
- [ ] Web collaboration platform

---

## Contributing

This project is in active development. If you work in VDC, construction technology, 
or Rust systems programming — contributions and feedback are welcome.

---

## License

Apache-2.0 — see [LICENSE](LICENSE)
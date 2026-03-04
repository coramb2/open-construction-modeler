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

Early development — core data model and project persistence complete.

**Working:**
- Construction object data model (trade, LOD, CSI code, phase, status)
- Object relationships (dependencies, sequencing, hosting, assemblies)
- Project save/load to `.ocm` format
- Round-trip JSON serialization

**In Progress:**
- IFC 4.3 import/export
- Geometry engine (Open CASCADE via Rust FFI)
- CLI interface

---

## Architecture
```
crates/
├── engine/   — Core data model, relationships, project persistence
├── ifc/      — IFC 4.3 parser and writer
└── app/      — CLI application binary
```

**Language:** Rust  
**Geometry Kernel:** Open CASCADE (planned)  
**Rendering:** wgpu / WebGPU (planned)  
**Scripting:** Python via PyO3 (planned)  

---

## Building
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/coramb2/open-construction-modeler.git
cd open-construction-modeler
cargo build

# Run tests
cargo test

# Run the app
cargo run
```

---

## Roadmap

- [ ] IFC 4.3 import/export
- [ ] NWC and DWG import
- [ ] Geometry engine integration
- [ ] Clash detection engine
- [ ] GPU viewport
- [ ] Python scripting layer
- [ ] One-click publish to collaboration platform
- [ ] Standalone clash analysis mode

---

## Contributing

This project is in early stages. If you work in VDC, construction technology, 
or Rust systems programming — contributions and feedback are welcome.

See `docs/research.md` for design decisions and user research.

---

## License

Apache-2.0 — see [LICENSE](LICENSE)
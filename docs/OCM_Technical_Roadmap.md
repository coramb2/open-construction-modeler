# Open Construction Modeler — Technical Roadmap
**Version 0.1 — March 2026**
**Repository:** https://github.com/coramb2/open-construction-modeler

---

## Executive Summary

Open Construction Modeler (OCM) is an open-source, construction-first 3D modeling and coordination platform built in Rust. It is designed to replace the painful multi-tool workflow (Revit → Navisworks → coordination → repeat) that VDC coordinators face daily. The system is architected as two interconnected products: a high-performance desktop modeling engine and a web-based collaboration platform.

---

## Part 1 — Desktop Modeling Engine

### Architecture Overview

```
open-construction-modeler/
├── crates/
│   ├── engine/       — Core data model, serialization, project persistence
│   ├── ifc/          — IFC parser and geometry extraction
│   └── app/          — CLI binary for testing and scripting
├── src-tauri/        — Tauri desktop application backend (Rust)
├── frontend/         — React + TypeScript + Three.js UI
└── docs/             — Architecture decisions and research
```

**Language:** Rust (engine, backend)
**UI Framework:** Tauri 2 + React + TypeScript
**3D Rendering:** Three.js (WebGL)
**Geometry Kernel:** Open CASCADE Technology via FFI (planned)
**Build System:** Cargo workspace (multi-crate)
**Serialization:** serde + serde_json
**Project Format:** .ocm (JSON-based, human-readable)

---

## Completed Work (Sprint 0–5)

### Sprint 0 — Workspace Foundation
- Rust 1.93 toolchain installed and configured
- Cargo workspace with three library/binary crates
- Apache 2.0 license, .gitignore, README
- GitHub repository initialized with clean commit history

### Sprint 1 — Core Data Model
**Files:** `crates/engine/src/`

The construction object schema is the foundation of the entire system. Every physical element in a construction project is represented as a `ConstructionObject`:

```
ConstructionObject
├── id: UUID (v4, auto-generated)
├── name: String
├── trade: Trade (enum)
├── lod: LodLevel (enum)
├── csi_code: String
├── phase: String
├── status: ConstructionStatus (enum)
├── approval_status: ApprovalStatus (enum)
├── geometry_ref: Option<String>
└── relations: Relations
```

**Trade enum:** Structural, Mechanical, Electrical, Plumbing, Civil, Architectural, FireProtection, Other(String)

**LOD enum:** Lod100, Lod200, Lod300, Lod350, Lod400, Lod500

**Status enum:** NotStarted, InProgress, Fabricating, Installed, Inspected, Complete

**ApprovalStatus enum:** Draft, InReview, Approved, Rejected, Superseded

**Relations struct:**
```
Relations
├── depends_on: Vec<UUID>        — installation dependencies
├── sequenced_after: Vec<UUID>   — scheduling dependencies (4D foundation)
├── hosted_by: Option<UUID>      — physical host object
└── assembly_parent: Option<UUID> — parent assembly
```

Key design decisions:
- Relationships are first-class data, not geometric constraints
- No separate architectural/structural/MEP model concept — one unified model with trade views
- `geometry_ref` is intentionally decoupled from metadata — geometry loads lazily
- Duplicate prevention enforced on dependency and sequencing lists

### Sprint 2 — Project Persistence + CLI
**Files:** `crates/engine/src/project.rs`, `crates/app/src/main.rs`

- `Project` struct: `HashMap<UUID, ConstructionObject>` container
- Save/load to `.ocm` file format (pretty-printed JSON)
- Full round-trip serialization verified by test
- CLI binary with commands: `new`, `list`, `add`, `filter`, `status`, `import`
- Graceful error handling via `anyhow` (no panics on bad input)
- Trade filtering working end-to-end

### Sprint 3 — IFC Import
**Files:** `crates/ifc/src/parser.rs`

- IFC STEP file reader (ISO-10303-21 format)
- Entity type detection for: IFCWALL, IFCWALLSTANDARDCASE, IFCSLAB, IFCBEAM, IFCCOLUMN, IFCDOOR, IFCWINDOW, IFCSTAIR, IFCDUCT, IFCPIPE
- Automatic trade classification from entity type
- GUID and name extraction via quote-split parsing
- Tested against FZK Haus sample file (IFC4, ArchiCAD export): 70 objects imported correctly
- Default LOD200 assigned to all IFC imports (design intent baseline)
- `import` CLI command added

### Sprint 4 — Desktop Application (Tauri + React)
**Files:** `src-tauri/`, `frontend/`

- Tauri 2.0 desktop application framework
- React 18 + TypeScript frontend
- Tailwind CSS for styling
- Three-panel CAD tool layout:
  - Left: Object tree (scrollable, trade + LOD display)
  - Center: Three.js 3D viewport
  - Right: Object inspector (all metadata fields)
- Three.js scene with WebGL renderer, perspective camera, ambient + directional lighting
- Grid helper for spatial reference
- Trade-color-coded geometry (blue = Structural, gray = Architectural, orange = Mechanical)
- OrbitControls: mouse-drag orbit, scroll zoom, pan
- Viewport resize handling

### Sprint 5 — Backend Integration + File Dialog
**Files:** `src-tauri/src/lib.rs`, `frontend/src/App.tsx`, `frontend/src/Viewport.tsx`

- Tauri command bridge: `load_project` and `get_project_path`
- React frontend calls Rust backend via `invoke()`
- Real `.ocm` project data rendered in Three.js viewport
- Three.js raycasting for 3D object selection (click box → highlights in sidebar)
- Bidirectional selection: sidebar click highlights viewport, viewport click highlights sidebar
- Native file open dialog via `tauri-plugin-dialog`
- File type routing: `.ocm` → JSON deserialize, `.ifc` → IFC parser → temporary project
- 8 passing unit tests across engine and ifc crates

---

## Test Coverage

| Crate | Tests | Coverage |
|-------|-------|----------|
| engine | 6 | ConstructionObject, serialization, round-trip, Project CRUD, Relations dedup, sequencing |
| ifc | 2 | Empty file parse, wall entity extraction |
| app | 0 | Covered by integration (CLI commands) |

---

## In Progress (Sprint 6)

### IFC Geometry Extraction

**Problem:** Current IFC parser extracts metadata but discards geometry. Objects render as uniform placeholder boxes regardless of real-world dimensions and position.

**Approach — Layered geometry pipeline:**

```
Layer 1 — IfcIndex
  HashMap<u32, String> — entire file indexed by entity ID
  Enables O(1) reference resolution

Layer 2 — Reference Resolver
  Follow #id chains: entity → placement → coordinates
  Handle nested IFCLOCALPLACEMENT chains

Layer 3 — Placement Extractor
  IFCLOCALPLACEMENT → world matrix
  IFCAXIS2PLACEMENT3D → position + rotation
  IFCCARTESIANPOINT → XYZ coordinates

Layer 4 — Profile Extractor
  IFCARBITRARYCLOSEDPROFILEDEF → closed polygon
  IFCRECTANGLEPROFILEDEF → width + depth
  IFCCIRCLEPROFILEDEF → radius

Layer 5 — Solid Extractor
  IFCEXTRUDEDAREASOLID → profile + depth → final dimensions
  IFCFACETEDBREP → triangulated mesh (advanced)

Layer 6 — Three.js Renderer
  BoxGeometry for rectangular solids (walls, slabs, columns)
  ExtrudeGeometry for arbitrary profiles
  Correct world position and rotation applied
```

**Status:** `IfcIndex` struct designed, `geometry.rs` file created, implementation starting.

**Expected output:** Walls render as flat rectangles at correct position, beams as long thin boxes, slabs as large flat planes — all at real-world scale and coordinates.

---

## Planned Sprints

### Sprint 7 — Trade Filter Toggles
- Show/hide trades in viewport via toggle buttons
- Filter state persisted in React
- Mesh visibility updated without scene rebuild
- Useful for: MEP coordination, structural review, architectural review

### Sprint 8 — Clash Detection Engine
**Core algorithm:**
```
For each object A:
  For each object B (where B.trade != A.trade):
    Compute AABB (axis-aligned bounding box) overlap
    If overlap exists: record ClashResult { a_id, b_id, overlap_volume, severity }
```

- Hard clash: solid geometry intersection
- Soft clash (clearance): configurable buffer distance
- Clash report: serializable list of ClashResult objects
- Visual highlight: clashing objects rendered in red
- CLI command: `cargo run -- clash`

This directly addresses the primary user pain point: "One button Revit to Navisworks" — our clash detection runs natively without export.

### Sprint 9 — DWG/DXF Import
- Civil site data import (addresses civil coordination pain point)
- ODA File Converter integration or open DXF parser
- Extract: layers, entities, coordinates, elevations
- Map DWG layers to Trade classification
- Site objects imported as Civil trade

### Sprint 10 — Lifecycle + Change Tracking
```
Lifecycle
├── created_at: DateTime
├── modified_at: DateTime
├── reviewed_by: Option<String>
├── review_date: Option<DateTime>
└── change_history: Vec<ChangeEvent>

ChangeEvent
├── timestamp: DateTime
├── field: String
├── old_value: String
└── new_value: String
```

- Object-level audit trail
- Model health score based on LOD compliance and status completeness

### Sprint 11 — Python Scripting Layer
- PyO3 bindings exposing engine crate to Python
- Scriptable object creation, filtering, bulk status updates
- Plugin API for custom validation rules
- Entry point for AI-assisted workflows

### Sprint 12 — Git-Style Model Versioning
- Object-level diffs (which fields changed, which objects added/removed)
- Branch support for design alternatives
- Merge workflow for multi-team coordination
- Change history timeline in UI

---

## Part 2 — Web Collaboration Platform (Planned)

### Architecture

```
Frontend:   Next.js 14 + React + Tailwind
Backend:    Node.js or Go + REST + GraphQL
Database:   PostgreSQL (row-level security from day one)
Search:     Elasticsearch
Storage:    S3-compatible object storage
Queue:      Redis
CDN:        Edge caching for model assets
Auth:       Auth.js / Lucia — OAuth 2.0 + 2FA
```

### Core Features (Planned)

**Model Repository**
- Upload IFC, .ocm, glTF files
- Version control with diff viewer
- Browser-based Three.js model viewer
- Metadata search: trade, LOD, phase, CSI code
- Comment threads on model objects

**Knowledge Platform**
- Markdown documentation with embedded model viewer
- Version-controlled tutorials
- SEO-indexed for construction search terms

**Script + Plugin Marketplace**
- Git-style repos for Rust/Python plugins
- Version history, releases, ratings
- Dependency metadata

**Forum + Q&A**
- Stack Overflow-style threaded discussions
- Reputation system, accepted answers
- Moderation tools

**Integration with Desktop App**
- One-click publish from desktop to platform
- Shared authentication (API tokens)
- Cloud validation engine
- Webhook support for CI/CD model validation

---

## File Format Support Matrix

| Format | Read | Write | Priority | Notes |
|--------|------|-------|----------|-------|
| .ocm | ✅ | ✅ | Core | Native project format |
| IFC 4.x | ✅ partial | Planned | High | Geometry extraction in progress |
| IFC 2x3 | ✅ partial | — | High | Same parser, tested |
| DWG/DXF | Planned | — | Medium | Civil site data |
| NWC | Planned | — | High | Navisworks — proprietary |
| RVT | Via IFC | — | Medium | Revit export → IFC handoff |
| glTF | Planned | Planned | Medium | Web viewer, AR/VR |
| BCF | Planned | Planned | Medium | Issue tracking standard |

---

## Long-Term Technical Vision

**4D Modeling (Schedule Integration)**
- Link ConstructionObject.phase to CPM schedule
- Animate construction sequence in viewport
- Integrate with Primavera P6 / MS Project export formats

**5D Modeling (Cost Integration)**
- CSI code → cost database lookup
- Quantity takeoff from geometry
- Cost report generation

**AI-Assisted Coordination**
- Clash resolution suggestions based on trade rules
- Automated LOD compliance checking
- Anomaly detection in model health

**Cloud Rendering**
- Offload large model rendering to server
- Stream viewport frames to thin client
- Enable browser-only access without desktop install

**AR/VR Viewer**
- glTF export for Unity/Unreal integration
- WebXR viewer in browser
- QR code model placement on site

**Enterprise Deployment**
- On-premises installation option
- LDAP/Active Directory authentication
- Air-gapped network support (critical for government/defense projects)

---

## Technical Risk Register

| Risk | Severity | Mitigation |
|------|----------|------------|
| IFC geometry complexity | High | Layered extraction approach, test against multiple real files |
| Open CASCADE Rust FFI stability | Medium | Evaluate alternative: truck (pure Rust BREP) |
| NWC format is proprietary | High | Focus on IFC as primary, NWC via conversion tools |
| Large model performance (10GB+) | High | Lazy geometry loading, progressive streaming, instancing |
| Tauri WebView platform differences | Low | Test on Windows early, Linux primary dev target |
| serde_json schema evolution | Medium | Version field in .ocm format, migration scripts |

---

## Open Source Strategy

**License:** Apache 2.0
- Allows commercial use with attribution
- Protects contributors from patent claims
- Compatible with most corporate open source policies
- Preferred over MIT for an ecosystem play

**Contributor Onboarding Path:**
1. Good first issues labeled in GitHub
2. Architecture decision records in `docs/`
3. User research documented in `docs/research.md`
4. All design decisions explained in commit messages
5. No "magic" — every architectural choice is documented

**Differentiation from incumbents:**
- Revit: Closed source, architect-first, expensive, Windows only
- Navisworks: Coordination only, no modeling, expensive
- Tekla: Structural only, expensive
- FreeCAD: General purpose, not construction-aware
- OCM: Open source, construction-first, cross-platform, unified modeling + coordination

---

*Document generated: March 2026*
*Next update: After Sprint 6 completion (IFC geometry extraction)*

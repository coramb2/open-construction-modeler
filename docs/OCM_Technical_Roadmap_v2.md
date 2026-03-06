# Open Construction Modeler — Technical Roadmap v2
**Updated: March 2026**
**Repository:** https://github.com/coramb2/open-construction-modeler

---

## Core Design Principle

> "Don't add to the problem. Be the solution."

Every feature decision must pass this test:
- Does this reduce the number of tools a VDC coordinator needs?
- Does this eliminate a manual handoff step?
- Does this make someone's workday meaningfully simpler?

If the answer is no to all three, we don't build it.

---

## The Problem We're Solving

The construction industry runs on fragmented software. A typical VDC coordinator touches:

| Tool | Purpose | Problem |
|------|---------|---------|
| Revit | Modeling | Slow, crashes on large models, architect-first |
| Navisworks | Clash detection | Manual export from Revit, separate tool |
| Bluebeam | Document markup | Disconnected from model |
| Procore | Project management | No model awareness |
| Primavera / MS Project | Scheduling | No connection to model objects |
| PlanSwift / Destini | Cost estimating | Manual quantity entry |
| Teams / Outlook | Communication | Issues live in email, not in model |
| Excel | Everything else | The universal duct tape |

The result: the same information lives in 6-8 different places simultaneously, gets out of sync constantly, and requires manual labor to reconcile. Every tool handoff is a point of failure.

**OCM's answer:** One model-centric platform that connects to everything else. Not a replacement for all tools — a connective layer that makes them work together.

---

## Strategic Vision: The Construction Operating System

OCM is not a modeling tool that also does clash detection.

OCM is the **construction project operating system** — the model-centric source of truth that every other tool feeds into and reads from.

```
                    ┌─────────────────────────────┐
                    │   OCM — Project Hub          │
                    │                              │
   Revit ──IFC────▶│  Model  │ Clashes │ Issues   │
   Civil 3D ──────▶│  Objects│ Status  │ Schedule │◀──── Primavera
   Procore ────────▶│  Costs  │ Docs    │ RFIs     │
   Bluebeam ───────▶│                              │────▶ Procore
                    │  One source of truth         │────▶ Teams/Slack
                    └─────────────────────────────┘
```

The goal is not to replace Procore or Primavera. The goal is to be the model layer that gives all of them context they currently lack.

---

## Completed Work (Sprints 0–6 partial)

### Foundation (Sprint 0)
- Rust 1.93 workspace with Cargo multi-crate architecture
- Apache 2.0 license, GitHub repository
- CI-ready structure

### Core Data Model (Sprint 1)
```
ConstructionObject
├── id: UUID
├── name, trade, lod, csi_code, phase
├── status: NotStarted → Complete
├── approval_status: Draft → Approved
├── position: Option<[f64; 3]>      ← real-world coordinates
├── dimensions: Option<[f64; 3]>    ← real-world dimensions
└── relations
    ├── depends_on: Vec<UUID>
    ├── sequenced_after: Vec<UUID>
    ├── hosted_by: Option<UUID>
    └── assembly_parent: Option<UUID>
```

### Project Persistence (Sprint 2)
- .ocm project format (JSON, human-readable)
- Full round-trip serialization
- CLI: new, list, add, filter, status, import

### IFC Import (Sprint 3)
- IFC 4.x and 2x3 parser
- 70 objects imported from FZK Haus sample
- Trade auto-classification from entity type

### Desktop Application (Sprint 4)
- Tauri 2.0 + React + TypeScript
- Three-panel CAD layout
- Three.js WebGL viewport

### Backend Integration (Sprint 5)
- Tauri command bridge (Rust ↔ React)
- Raycasting object selection
- Native file open dialog
- .ocm and .ifc file routing

### IFC Geometry Extraction (Sprint 6 — in progress)
- IfcIndex: O(1) entity lookup by ID
- Reference resolver: follow #id chains
- Placement extractor: IFCLOCALPLACEMENT → XYZ
- Extrusion depth extractor: IFCEXTRUDEDAREASOLID → height
- Real dimensions rendering in viewport
- Trade filter toggles with color coding
- Noise object filtering
- Z-up to Y-up coordinate conversion

**14 passing tests across engine and ifc crates**

---

## Active Sprint: IFC Geometry (Sprint 6 continued)

### Remaining: World Matrix Resolver
Full parent chain traversal for correct world coordinates:

```
Layer 7 — World Matrix
  Walk IFCLOCALPLACEMENT parent chain recursively
  Compute 4x4 transform matrix at each level
  Multiply matrices: world = parent_world × local
  Apply to Three.js mesh position + rotation

Expected result: objects form recognizable building footprint
```

---

## Planned Sprints — Year 1

### Sprint 7 — Clash Detection Engine
**This is the core differentiator. This is what replaces Navisworks.**

```rust
struct ClashResult {
    object_a: Uuid,
    object_b: Uuid,
    clash_type: ClashType,  // Hard, Soft, Clearance
    overlap_volume: f64,
    severity: ClashSeverity,
    position: [f64; 3],
}
```

Algorithm:
1. Broad phase: AABB (axis-aligned bounding box) overlap test
2. Narrow phase: geometry-level intersection for confirmed candidates
3. Soft clash: configurable clearance buffer per trade pair
4. Report: JSON export, BCF export (industry standard issue format)

UI: clashing objects highlight red, clash count badge, clash list panel

**Business value:** "Run clash detection without opening Navisworks" is the demo that gets VDC coordinators to pay attention.

### Sprint 8 — BCF Issue Export
BCF (BIM Collaboration Format) is the open standard for model issues.
- Export clash results as BCF 2.1
- Import BCF issues from other tools
- Link issues to model objects
- Issue status tracking in OCM

**Why this matters:** BCF is how Revit, Navisworks, and Procore exchange issues. Supporting it means OCM speaks the industry's native issue language without requiring anyone to change their workflow.

### Sprint 9 — Procore Integration
Procore has an open API and is explicitly built to integrate with other platforms. This is our first external integration.

Capabilities:
- Sync OCM clash results → Procore RFIs automatically
- Pull Procore project roster → OCM team members
- Link OCM objects → Procore cost codes
- Push model status updates → Procore punch list

Authentication: OAuth 2.0 (Procore standard)
Implementation: Tauri backend makes authenticated API calls

**Why Procore first:** It's the most widely adopted construction management platform, has a well-documented open API, and offers a free developer account. This integration alone justifies OCM adoption for any team already on Procore.

### Sprint 10 — DWG/DXF Civil Import
Address the civil coordination pain point directly.
- Parse DWG/DXF via open format libraries
- Extract: layers, polylines, elevations, survey points
- Map DWG layers → Trade.Civil classification
- Site objects appear in OCM viewport at correct coordinates
- Coordinate with IFC building model automatically

### Sprint 11 — Schedule Integration (4D)
Connect model objects to construction schedule.
- Import Primavera P6 XML or MS Project XML
- Link ConstructionObject.phase → schedule activity
- 4D simulation: animate construction sequence in viewport
- Critical path visualization on model objects
- Schedule delay impact: which objects are affected?

**Why this matters:** 4D modeling is currently a separate, expensive workflow. Building it natively into OCM at no additional cost is a significant competitive advantage.

### Sprint 12 — Cost Integration (5D)
Connect model objects to cost data.
- CSI code → unit cost database lookup
- Automatic quantity takeoff from geometry dimensions
- Cost report by trade, phase, and CSI division
- Budget vs actual tracking linked to object status
- Export to Excel (required — Excel is universal in construction finance)

**Why Excel export:** Excel is the universal language of construction finance. We don't fight this — we feed it.

### Sprint 13 — Python Scripting Layer
- PyO3 bindings exposing engine crate to Python
- Scriptable workflows: bulk status updates, custom validation, report generation
- Plugin API for custom clash rules
- Entry point for AI-assisted features

### Sprint 14 — Lifecycle + Change Tracking
```
ChangeEvent
├── timestamp: DateTime
├── author: String
├── field: String
├── old_value: String
└── new_value: String
```
- Full audit trail per object
- Model diff between versions
- Who changed what and when
- Feeds compliance documentation requirements

---

## Planned Sprints — Year 2

### Web Collaboration Platform (Part 2)
Only built after desktop validation is complete.

```
Frontend:  Next.js + React + Tailwind
Backend:   Rust (Axum) for consistency with desktop engine
Database:  PostgreSQL with row-level security
Storage:   S3-compatible object storage
Auth:      OAuth 2.0 + 2FA from day one
```

Features:
- Model repository with version history
- Browser-based Three.js viewer (reuse frontend code)
- One-click publish from desktop → platform
- Clash report sharing
- Team coordination dashboard
- Script/plugin marketplace

### Additional Integrations (Year 2+)
Prioritized by user demand, not by technical interest:

| Integration | Value | Complexity |
|-------------|-------|------------|
| Autodesk BIM 360 | High | High (proprietary) |
| Bluebeam | Medium | Medium |
| Primavera P6 direct | High | Medium |
| Microsoft Teams | Medium | Low (open API) |
| Slack | Medium | Low (open API) |
| QuickBooks / Sage | Medium | Medium |
| NWC import | High | Very High (proprietary) |

### AI-Assisted Features (Year 2+)
Only after core workflows are validated:
- Clash resolution suggestions based on trade rules
- Automated LOD compliance checking
- Schedule impact prediction
- Anomaly detection in model health scores

---

## Integration Philosophy

**We integrate with tools people already use. We don't replace them prematurely.**

The right sequence:
1. Be better at modeling + clash detection than the current workflow
2. Sync results to tools people already trust (Procore, BCF)
3. Gradually absorb workflows as trust is established
4. Never force migration — always offer export

This is how you avoid adding to the fragmentation problem. You become the thing that makes everything else work better, not the thing that demands people abandon what they know.

---

## File Format Support Matrix

| Format | Read | Write | Priority | Notes |
|--------|------|-------|----------|-------|
| .ocm | ✅ | ✅ | Core | Native format |
| IFC 4.x | ✅ partial | Planned | High | Geometry in progress |
| IFC 2x3 | ✅ partial | — | High | Same parser |
| BCF 2.1 | Planned | Planned | High | Issue exchange standard |
| DWG/DXF | Planned | — | Medium | Civil site data |
| Primavera XML | Planned | — | Medium | Schedule import |
| MS Project XML | Planned | — | Medium | Schedule import |
| glTF | Planned | Planned | Medium | Web viewer, AR/VR |
| NWC | Research | — | Low | Proprietary format |
| RVT | Via IFC | — | Low | Revit export handoff |
| Excel | — | Planned | High | Cost reports |
| PDF | — | Planned | Medium | Clash reports |

---

## Technical Risk Register

| Risk | Severity | Mitigation |
|------|----------|------------|
| IFC geometry complexity | High | Layered approach, test on 5+ real files |
| Procore API rate limits | Low | Cache responses, batch requests |
| NWC proprietary format | High | Accept IFC as primary, NWC via conversion |
| Large model performance | High | Lazy loading, progressive streaming |
| World matrix chain depth | Medium | Iterative resolver with cycle detection |
| serde schema evolution | Medium | Version field in .ocm, migration scripts |
| Windows WebView differences | Medium | Test on Windows every cycle |

---

## Open Source Strategy

**License:** Apache 2.0

**Why open source is the right business model here:**
- Construction firms distrust vendor lock-in after decades of Autodesk pricing
- Open source builds trust faster than any marketing
- Community contributions accelerate format support (DWG, NWC, etc.)
- Consulting and enterprise support are the revenue model, not the software itself

**Revenue paths:**
1. Paid consulting: custom IFC workflows, Procore integrations, clash rule sets
2. Enterprise support contracts: SLA, private deployment, custom features
3. Hosted platform (Part 2): storage, team features, API access tiers
4. Training and certification: VDC teams adopting OCM workflows

**What stays free forever:** the core engine, clash detection, IFC import/export, the desktop application. These are the foundation of trust.

---

## Competitive Positioning

| | OCM | Revit | Navisworks | Procore | FreeCAD |
|--|-----|-------|-----------|---------|---------|
| Open source | ✅ | ❌ | ❌ | ❌ | ✅ |
| Construction-first | ✅ | ❌ | Partial | ✅ | ❌ |
| Native clash detection | ✅ | ❌ | ✅ | ❌ | ❌ |
| IFC native | ✅ | Partial | Partial | ❌ | ✅ |
| Procore integration | Planned | ❌ | ❌ | N/A | ❌ |
| Schedule integration | Planned | Plugin | ❌ | ✅ | ❌ |
| Free | ✅ | ❌ | ❌ | ❌ | ✅ |
| Cross-platform | ✅ | Windows | Windows | Web | ✅ |

---

*Version 2 — March 2026*
*Next update: After Cycle 1 external validation (3 VDC tester responses)*
# User Research

## Primary User Profile

VDC Coordinator working daily in Revit, Navisworks, and Civil 3D.
Primary pain points surfaced through direct interview.

## Key Pain Points

### 1. Performance on Large Models
Revit crashes and slows significantly on large models. Root cause is eager 
loading — entire model loads into memory at once. Our architecture addresses 
this through lazy geometry loading and progressive streaming.

### 2. Civil Coordination
Importing and coordinating terrain/site data from Civil 3D is painful. 
Revit's toposurface tools handle DWG civil data poorly. We need first-class 
DWG/DXF site data import.

### 3. Revit to Navisworks Friction
The export/import cycle between Revit and Navisworks is manual and slow. 
One-click publish from modeling to clash review is a headline feature 
that directly addresses this.

### 4. Real-World File Formats
Daily formats in order of usage: NWC, RVT, DWG, IFC (least common).
IFC is the right open standard but cannot be the only supported format.
NWC and DWG import must be first-class citizens.

## Product Direction Insight

The tool does not need to be all-encompassing. A focused clash analysis 
tool that beats Navisworks on speed and simplicity could drive adoption 
toward the full modeling platform. Consider standalone clash viewer as 
an early release target.

## Feature Priorities

1. Fast, reliable core (performance first)
2. One-click publish/share workflow
3. NWC + DWG import alongside IFC
4. Standalone clash analysis mode
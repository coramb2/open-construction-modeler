# Open Construction Modeler

An open-source, construction-first 3D modeling engine built for VDC, coordination, and trade workflows.

## Architecture

- `crates/engine` — Core geometry and data model
- `crates/ifc` — IFC 4.3 parser and writer

## Building
```bash
cargo build
```

## License

Apache-2.0
```

**`LICENSE`** — go to this URL and paste the full text into a file called `LICENSE`:
```
https://www.apache.org/licenses/LICENSE-2.0.txt
```

**`.gitignore`** — at the project root:
```
/target
Cargo.lock
.DS_Store
*.swp
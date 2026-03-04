use anyhow::Result;
use engine::metadata::{LodLevel, Trade};
use engine::object::ConstructionObject;
use engine::project::Project;

fn main() -> Result<()> {
    println!("Open Construction Modeler");
    println!("--------------------------");

    // Create a new project
    let mut project = Project::new("First Project".to_string());
    println!("Created project: {}", project.name);

    // Add construction objects
    let slab = ConstructionObject::new(
        "Level 1 Slab".to_string(),
        Trade::Structural,
        LodLevel::LOD300,
        "03 30 00".to_string(),
        "Phase 1".to_string(),
    );

    let duct = ConstructionObject::new(
        "Main Supply Duct".to_string(),
        Trade::Mechanical,
        LodLevel::LOD300,
        "23 31 00".to_string(),
        "Phase 1".to_string(),
    );

    println!("Adding object: {}", slab.name);
    println!("Adding object: {}", duct.name);

    project.add_object(slab);
    project.add_object(duct);

    // Save to disk
    let path = "/tmp/first_project.ocm";
    project.save(path)?;
    println!("Project saved to {}", path);

    // Load from disk
    let loaded = Project::load(path)?;
    println!("Project loaded: {} ({} objects)", loaded.name, loaded.objects.len());

    Ok(())
}
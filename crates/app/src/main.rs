use anyhow::Result;
use clap::{Parser, Subcommand};
use engine::metadata::{ConstructionStatus, LodLevel, Trade};
use engine::object::ConstructionObject;
use engine::project::Project;
use ifc;

#[derive(Parser)]
#[command(name = "ocm")]
#[command(about = "Open Construction Modeler CLI")]
struct Cli {
    #[arg(short, long, default_value = "project.ocm")]
    project: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    New {
        name: String,
    },
    List,
    Add {
        name: String,
        #[arg(long)]
        trade: String,
        #[arg(long)]
        entity_type: Option<String>,
        #[arg(long)]
        lod: u32,
        #[arg(long)]
        csi: String,
        #[arg(long)]
        phase: String,
    },
    Filter {
        #[arg(long)]
        trade: String,
    },
    Status {
        id: String,
        status: String,
    },
    Import {
        file: String,
    },
}

fn parse_trade(s: &str) -> Trade {
    match s.to_lowercase().as_str() {
        "structural" => Trade::Structural,
        "mechanical" => Trade::Mechanical,
        "electrical" => Trade::Electrical,
        "plumbing" => Trade::Plumbing,
        "civil" => Trade::Civil,
        "architectural" => Trade::Architectural,
        "fireprotection" => Trade::FireProtection,
        other => Trade::Other(other.to_string()),
    }
}

fn parse_lod(n: u32) -> anyhow::Result<LodLevel> {
    match n {
        100 => Ok(LodLevel::Lod100),
        200 => Ok(LodLevel::Lod200),
        300 => Ok(LodLevel::Lod300),
        350 => Ok(LodLevel::Lod350),
        400 => Ok(LodLevel::Lod400),
        500 => Ok(LodLevel::Lod500),
        _ => anyhow::bail!("Invalid LOD level: {}. Valid values: 100, 200, 300, 350, 400, 500", n),
    }
}

fn parse_status(s: &str) -> ConstructionStatus {
    match s.to_lowercase().as_str() {
        "notstarted" => ConstructionStatus::NotStarted,
        "inprogress" => ConstructionStatus::InProgress,
        "fabricating" => ConstructionStatus::Fabricating,
        "installed" => ConstructionStatus::Installed,
        "inspected" => ConstructionStatus::Inspected,
        "complete" => ConstructionStatus::Complete,
        _ => ConstructionStatus::NotStarted, // Default to NotStarted for invalid input
    }
}

fn main() -> Result<()> {
     let cli = Cli::parse();

     match cli.command {
        Commands::New { name } => {
            let project = Project::new(name.clone());
            project.save(&cli.project)?;
            println!("Created project '{}' at {}", name, cli.project);
        }

        Commands::List => {
            let project = Project::load(&cli.project)?;
            println!("Project: {}", project.name);
            println!("{} object(s)\n", project.objects.len());
            for obj in project.objects.values() {
                println!(" [{}] {} - {:?} | {:?} | {} | {:?}",
                    obj.id, obj.name, obj.trade, obj.lod, obj.csi_code, obj.status);
            }
        }

        Commands::Add { name, trade,
             entity_type, lod, csi, phase } => {
            let mut project = Project::load(&cli.project)?;
            let obj = ConstructionObject::new(
                name.clone(),
                parse_trade(&trade),
                entity_type,
                parse_lod(lod)?,
                csi,
                phase,
            );
            println!("Added '{}' [{}]", obj.name, obj.id);
            project.add_object(obj);
            project.save(&cli.project)?;
        }

        Commands::Filter { trade } => {
            let project = Project::load(&cli.project)?;
            let trade_parsed = parse_trade(&trade);
            let filtered: Vec<_> = project.objects.values()
                .filter(|o| format!("{:?}", o.trade) == format!("{:?}", trade_parsed))
                .collect();
            println!("{} object(s) for trade {:?}\n", filtered.len(), trade_parsed);
            for obj in filtered {
                println!(" [{}] {} - {:?} | {:?} | {}",
                    obj.id, obj.name, obj.trade, obj.lod, obj.csi_code);
        }
     }

        Commands::Status { id, status } => {
            let mut project = Project::load(&cli.project)?;
            let uuid = uuid::Uuid::parse_str(&id)?;
            
            let updated_name = if let Some(obj) = project.objects.get_mut(&uuid) {
                obj.status = parse_status(&status);
                Some(obj.name.clone()) // extract name while we have the borrow
            } else {
                None
            };
            // mutable borrow is dropped here
            
            if let Some(name) = updated_name {
                project.save(&cli.project)?;
                println!("Updated '{}' status to {}", name, status);
            } else {
                println!("Object not found: {}", id);
            }
        }

        Commands::Import { file } => {
            let mut project = Project::load(&cli.project)?;
            let objects = ifc::parser::parse_ifc_file(&file)?;
            let count = objects.len();
            for obj in objects {
                project.add_object(obj);
            }
            project.save(&cli.project)?;
            println!("Imported {} objects from {}", count, file);
        }
    }
    Ok(())
}
use clap::{Parser, Subcommand};
use colored::*;
use memory_engine::{ProjectMemory, CodeParser, MemoryStorage};
use std::path::Path;

#[derive(Parser)]
#[command(name = "aimemoryengine")]
#[command(about = "AI Memory Engine for persistent project context")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize memory tracking for current project
    Init,
    /// Show memory statistics
    Status,
    /// Query project context
    Query { pattern: String },
    /// Analyze specific file
    Analyze { file_path: String },
    /// Reset project memory
    Reset,
}

fn get_db_path() -> anyhow::Result<String> {
    let current_dir = std::env::current_dir()?;
    let db_dir = current_dir.join(".aimemoryengine");

    // Create directory if it doesn't exist
    if !db_dir.exists() {
        std::fs::create_dir_all(&db_dir)?;
    }

    let db_path = db_dir.join("memory.db");
    Ok(db_path.to_string_lossy().to_string())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            println!("{}", "🧠 Initializing AI Memory Engine...".green());
            let current_dir = std::env::current_dir()?;
            let db_path = get_db_path()?;

            // Create storage and initialize empty memory
            let storage = MemoryStorage::new(&db_path)?;
            let memory = ProjectMemory::new(current_dir.to_string_lossy().to_string());
            storage.save_memory(&memory)?;

            println!("{}", "✅ Memory engine initialized!".green());
            println!("Project path: {}", current_dir.display());
            println!("Database: {}", db_path);
        }
        Commands::Status => {
            println!("{}", "📊 Memory Engine Status".blue().bold());
            let current_dir = std::env::current_dir()?;
            let db_path = get_db_path()?;

            if !Path::new(&db_path).exists() {
                println!("{}", "❌ Memory engine not initialized. Run 'aimemoryengine init' first.".red());
                return Ok(());
            }

            let storage = MemoryStorage::new(&db_path)?;
            let (entity_count, relationship_count, file_count) = storage.get_stats()?;

            println!("Project: {}", current_dir.display());
            println!("Database: {}", db_path);
            println!("Entities: {}", entity_count);
            println!("Relationships: {}", relationship_count);
            println!("Files tracked: {}", file_count);
        }
        Commands::Query { pattern } => {
            println!("{}", format!("🔍 Searching for: {}", pattern).yellow());
            let db_path = get_db_path()?;

            if !Path::new(&db_path).exists() {
                println!("{}", "❌ Memory engine not initialized. Run 'aimemoryengine init' first.".red());
                return Ok(());
            }

            let storage = MemoryStorage::new(&db_path)?;

            match storage.find_entities_by_name(&pattern) {
                Ok(entities) => {
                    if entities.is_empty() {
                        println!("No entities found matching '{}'", pattern);
                    } else {
                        println!("\n📋 Found {} entities:", entities.len());
                        for entity in entities {
                            println!("  {} {} in {} at line {}",
                                entity.entity_type.as_str(),
                                entity.name.green(),
                                entity.file_path.blue(),
                                entity.line_start
                            );
                        }
                    }
                }
                Err(e) => println!("❌ Error querying database: {}", e),
            }
        }
        Commands::Analyze { file_path } => {
            println!("{}", format!("🔬 Analyzing file: {}", file_path).cyan());

            let db_path = get_db_path()?;
            let storage = MemoryStorage::new(&db_path)?;
            let current_dir = std::env::current_dir()?;

            // Load existing memory or create new one
            let mut memory = if Path::new(&db_path).exists() {
                storage.load_memory(&current_dir.to_string_lossy())?
            } else {
                ProjectMemory::new(current_dir.to_string_lossy().to_string())
            };

            match CodeParser::new() {
                Ok(parser) => {
                    match parser.parse_file(&file_path) {
                        Ok((entities, relationships)) => {
                            println!("\n📊 Analysis Results:");
                            println!("Entities found: {}", entities.len());
                            println!("Relationships found: {}", relationships.len());

                            if !entities.is_empty() {
                                println!("\n🔍 Entities:");
                                for entity in &entities {
                                    println!("  {} {} at line {}",
                                        entity.entity_type.as_str(),
                                        entity.name.green(),
                                        entity.line_start
                                    );

                                    // Add entity to memory
                                    memory.add_entity(entity.clone());
                                }

                                // Add relationships to memory
                                for relationship in relationships {
                                    memory.add_relationship(relationship);
                                }

                                // Save updated memory to database
                                storage.save_memory(&memory)?;
                                println!("\n💾 {}", "Memory updated and saved!".green());
                            }
                        }
                        Err(e) => println!("❌ Error parsing file: {}", e),
                    }
                }
                Err(e) => println!("❌ Error creating parser: {}", e),
            }
        }
        Commands::Reset => {
            println!("{}", "🗑️  Resetting project memory...".red());
            let db_path = get_db_path()?;

            if Path::new(&db_path).exists() {
                std::fs::remove_file(&db_path)?;
                println!("{}", "✅ Memory database deleted successfully!".green());
            } else {
                println!("{}", "ℹ️  No memory database found to reset.".yellow());
            }
        }
    }

    Ok(())
}

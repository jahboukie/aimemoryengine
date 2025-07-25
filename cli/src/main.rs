use clap::{Parser, Subcommand};
use colored::*;
use memory_engine::{ProjectMemory, CodeParser};

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            println!("{}", "🧠 Initializing AI Memory Engine...".green());
            let current_dir = std::env::current_dir()?;
            let _memory = ProjectMemory::new(current_dir.to_string_lossy().to_string());
            println!("{}", "✅ Memory engine initialized!".green());
            println!("Project path: {}", current_dir.display());
        }
        Commands::Status => {
            println!("{}", "📊 Memory Engine Status".blue().bold());
            let current_dir = std::env::current_dir()?;
            let memory = ProjectMemory::new(current_dir.to_string_lossy().to_string());
            let stats = memory.get_stats();
            
            println!("Project: {}", stats.project_path);
            println!("Entities: {}", stats.entity_count);
            println!("Relationships: {}", stats.relationship_count);
            println!("Files tracked: {}", stats.file_count);
        }
        Commands::Query { pattern } => {
            println!("{}", format!("🔍 Searching for: {}", pattern).yellow());
            // TODO: Implement actual querying
            println!("Query functionality coming soon...");
        }
        Commands::Analyze { file_path } => {
            println!("{}", format!("🔬 Analyzing file: {}", file_path).cyan());

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
                                }
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
            // TODO: Implement memory reset
            println!("Reset functionality coming soon...");
        }
    }

    Ok(())
}

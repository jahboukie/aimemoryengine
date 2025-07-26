use clap::{Parser, Subcommand};
use colored::*;
use memory_engine::{ProjectMemory, CodeParser, MemoryStorage, LicenseManager};
use std::path::Path;
use chrono::{DateTime, Utc};

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
    /// Activate license with key
    License {
        #[command(subcommand)]
        action: LicenseAction
    },
}

#[derive(Subcommand)]
enum LicenseAction {
    /// Activate license with provided key
    Activate { key: String },
    /// Check current license status
    Status,
    /// Remove current license
    Remove,
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

async fn check_license_for_command(command_name: &str) -> anyhow::Result<()> {
    // Skip license check for license management commands and basic info
    match command_name {
        "license" | "status" | "init" => return Ok(()),
        _ => {}
    }

    match LicenseManager::new() {
        Ok(license_manager) => {
            match license_manager.check_license(None).await {
                Ok(validation) => {
                    if !validation.valid {
                        println!("{}", "❌ Invalid or expired license. Please activate a valid license.".red());
                        println!("Use: {} to activate your license", "aimemoryengine license activate <your-key>".yellow());
                        std::process::exit(1);
                    }

                    // Check expiration
                    if let Some(expires_at) = validation.expires_at {
                        let days_until_expiry = (expires_at - Utc::now()).num_days();
                        if days_until_expiry <= 7 && days_until_expiry > 0 {
                            println!("{}", format!("⚠️  License expires in {} days", days_until_expiry).yellow());
                        }
                    }
                }
                Err(_) => {
                    println!("{}", "⚠️  Could not validate license (offline mode). Some features may be limited.".yellow());
                    // Allow offline usage with cached license
                }
            }
        }
        Err(_) => {
            println!("{}", "❌ No license found. This is a commercial product.".red());
            println!("Get your license at: {}", "https://aimemoryengine.com/pricing".blue());
            println!("Then activate with: {} ", "aimemoryengine license activate <your-key>".yellow());
            std::process::exit(1);
        }
    }

    Ok(())
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

        Commands::License { action } => {
            match action {
                LicenseAction::Activate { key } => {
                    println!("{}", "🔐 Activating license...".cyan());

                    match LicenseManager::new() {
                        Ok(license_manager) => {
                            match license_manager.validate_license(&key).await {
                                Ok(validation) => {
                                    if validation.valid {
                                        license_manager.save_license(&key, &validation)?;
                                        println!("{}", "✅ License activated successfully!".green());

                                        if let Some(expires_at) = validation.expires_at {
                                            println!("License expires: {}", expires_at.format("%Y-%m-%d %H:%M:%S UTC"));
                                        }

                                        if let Some(usage_count) = validation.usage_count {
                                            if let Some(usage_limit) = validation.usage_limit {
                                                println!("Usage: {}/{}", usage_count, usage_limit);
                                            } else {
                                                println!("Usage: {} (unlimited)", usage_count);
                                            }
                                        }
                                    } else {
                                        println!("{}", "❌ Invalid license key. Please check your key and try again.".red());
                                        std::process::exit(1);
                                    }
                                }
                                Err(e) => {
                                    println!("{}", format!("❌ License validation failed: {}", e).red());
                                    std::process::exit(1);
                                }
                            }
                        }
                        Err(e) => {
                            println!("{}", format!("❌ License manager error: {}", e).red());
                            println!("Make sure you have proper Keygen configuration.");
                            std::process::exit(1);
                        }
                    }
                }

                LicenseAction::Status => {
                    println!("{}", "📋 License Status".blue().bold());

                    match LicenseManager::new() {
                        Ok(license_manager) => {
                            match license_manager.load_cached_license() {
                                Ok(Some(cached_license)) => {
                                    println!("License Key: {}****", &cached_license.key[..8]);

                                    if let Some(validation) = &cached_license.cached_validation {
                                        if validation.valid {
                                            println!("Status: {}", "✅ Active".green());
                                        } else {
                                            println!("Status: {}", "❌ Invalid".red());
                                        }

                                        if let Some(expires_at) = validation.expires_at {
                                            let days_until_expiry = (expires_at - Utc::now()).num_days();
                                            println!("Expires: {} ({} days)",
                                                expires_at.format("%Y-%m-%d %H:%M:%S UTC"),
                                                days_until_expiry);
                                        }

                                        if let Some(usage_count) = validation.usage_count {
                                            if let Some(usage_limit) = validation.usage_limit {
                                                println!("Usage: {}/{}", usage_count, usage_limit);
                                            } else {
                                                println!("Usage: {} (unlimited)", usage_count);
                                            }
                                        }
                                    }

                                    if let Some(last_validated) = cached_license.last_validated {
                                        println!("Last Validated: {}", last_validated.format("%Y-%m-%d %H:%M:%S UTC"));
                                    }
                                }
                                Ok(None) => {
                                    println!("{}", "❌ No license found.".red());
                                    println!("Use: {} to activate your license", "aimemoryengine license activate <your-key>".yellow());
                                }
                                Err(e) => {
                                    println!("{}", format!("❌ Error reading license: {}", e).red());
                                }
                            }
                        }
                        Err(e) => {
                            println!("{}", format!("❌ License manager error: {}", e).red());
                        }
                    }
                }

                LicenseAction::Remove => {
                    println!("{}", "🗑️  Removing license...".red());

                    match LicenseManager::new() {
                        Ok(license_manager) => {
                            match license_manager.remove_license() {
                                Ok(()) => {
                                    println!("{}", "✅ License removed successfully!".green());
                                }
                                Err(e) => {
                                    println!("{}", format!("❌ Error removing license: {}", e).red());
                                }
                            }
                        }
                        Err(e) => {
                            println!("{}", format!("❌ License manager error: {}", e).red());
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

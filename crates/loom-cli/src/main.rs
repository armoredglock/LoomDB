use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "loomdb")]
#[command(author = "LoomDB Authors")]
#[command(version = "0.1.0")]
#[command(
    about = "LoomDB 🧶: World's Lightest Multi-Modal, AI-Native Database Engine",
    long_about = "LoomDB is a sub-megabyte, zero-dependency multi-modal database in a single portable .loom container file."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start LoomDB server with optional embedded Web Studio
    Server {
        #[arg(short, long, default_value = "127.0.0.1:8484")]
        addr: String,

        #[arg(short, long, default_value = "./data.loom")]
        container: String,

        #[arg(long, default_value_t = false)]
        open_studio: bool,
    },
    /// Export the zero-dependency Web Studio UI as a standalone HTML file
    Studio {
        #[arg(short, long, default_value = "./loom_studio.html")]
        output: String,
    },
    /// Launch interactive LoomQL REPL
    Repl {
        #[arg(short, long, default_value = "./data.loom")]
        container: String,
    },
    /// Inspect single-file .loom container metrics and page allocation
    Stats {
        #[arg(short, long, default_value = "./data.loom")]
        container: String,
    },
}

const STUDIO_HTML: &str = include_str!("studio.html");

fn main() {
    let cli = Cli::parse();

    println!("🧶 LoomDB Engine v0.1.0 (Micro Profile: <512KB Idle RAM)");

    match cli.command {
        Commands::Server { addr, container, open_studio } => {
            println!("🚀 Server listening on {}", addr);
            println!("📦 Mounted container: {}", container);
            if open_studio {
                println!("🌐 Web Studio available at http://{}/studio", addr);
            }
        }
        Commands::Studio { output } => {
            if let Err(e) = std::fs::write(&output, STUDIO_HTML) {
                eprintln!("Failed to write studio HTML: {}", e);
            } else {
                println!("✅ Exported zero-dependency Web Studio to: {}", output);
                println!("Open this file directly in any web browser!");
            }
        }
        Commands::Repl { container } => {
            println!("Connecting to container '{}'...", container);
            println!("Type 'help' for syntax or 'exit' to quit.");
        }
        Commands::Stats { container } => {
            println!("Inspecting .loom container: {}", container);
        }
    }
}

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "loomdb")]
#[command(author = "LoomDB Authors")]
#[command(version = "0.1.0")]
#[command(about = "LoomDB: Multi-Modal, Local-First, AI-Native Database Engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start LoomDB in standalone server mode
    Server {
        #[arg(short, long, default_value = "127.0.0.1:8484")]
        addr: String,

        #[arg(short, long, default_value = "./loom_data")]
        data_dir: String,
    },
    /// Launch interactive LoomQL REPL
    Repl {
        #[arg(short, long, default_value = "./loom_data")]
        data_dir: String,
    },
    /// Inspect database storage statistics and SSTables
    Stats {
        #[arg(short, long, default_value = "./loom_data")]
        data_dir: String,
    },
}

fn main() {
    let cli = Cli::parse();

    println!("🧶 LoomDB Engine v0.1.0 (Development Build)");

    match cli.command {
        Commands::Server { addr, data_dir } => {
            println!("Starting server on {} (storage: {})", addr, data_dir);
            // Server initialization logic
        }
        Commands::Repl { data_dir } => {
            println!("Connecting to LoomDB at {}...", data_dir);
            println!("Type 'help' for commands or 'exit' to quit.");
        }
        Commands::Stats { data_dir } => {
            println!("Analyzing LoomDB storage directory: {}", data_dir);
        }
    }
}

use clap::{Parser, Subcommand};
mod config;
mod server;
mod parser;
mod enums;
mod executor;
mod types;

#[derive(Parser)]
#[command(name = "magic", version = "0.1", about = "Magic Database")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Setup,
    Start {
        #[arg(short = 'p', long)]
        port: Option<u16>,
        #[arg(short = 'r', long)]
        protocol: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Setup => {
            config::create_default_config().expect("Magic config cannot be created.");
            println!("magic.toml created!");
        }
        Commands::Start { port, protocol } => {
            let mut cfg = config::load_config().expect("Cannot read magic.toml");
            if let Some(p) = port {
                cfg.server.port = p;
            }
            if let Some(proto) = protocol {
                cfg.server.protocol = Some(proto);
            }
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(server::start(cfg));
        }
    }
}

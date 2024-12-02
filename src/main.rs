use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Enable Debug Mode
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    std::env::set_var("RUST_LOG", "warn");
    if args.debug {
        std::env::set_var("RUST_LOG", "debug");
    }

    let session = match bluer::Session::new().await {
        Ok(session) => session,
        Err(e) => {
            log::error!("Failed to create session: {}", e);
            std::process::exit(1);
        }
    };
}
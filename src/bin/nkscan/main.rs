use clap::Parser;
use cli::Cli;
use nkscan::device;
use tracing_subscriber::EnvFilter;

mod cli;
mod io;
mod mono;
mod scan;

fn main() -> anyhow::Result<()> {
    // Set up logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    // Perform the requested CLI action
    match Cli::parse().action {
        cli::Action::List => {
            let devs = device::list();
            println!("Attached scanners:");
            devs.iter().for_each(|x| println!("{x}"));
        }
        cli::Action::Scan(args) => scan::run(args)?,
    }

    // Donezo
    Ok(())
}

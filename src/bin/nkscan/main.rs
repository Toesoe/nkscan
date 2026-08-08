use clap::Parser;
use cli::Cli;
use nkscan::device;

mod cli;
mod io;
mod mono;
mod scan;

fn main() -> anyhow::Result<()> {
    // Set up logging to stderr
    let subscriber = tracing_subscriber::fmt().with_writer(std::io::stderr);
    match tracing_subscriber::EnvFilter::try_from_default_env() {
        Ok(filter) => subscriber.with_env_filter(filter).init(),
        Err(_) => subscriber
            .with_env_filter(tracing_subscriber::EnvFilter::new("info"))
            .init(),
    }

    // Perform the requested CLI action
    match Cli::parse().action {
        cli::Action::List => {
            let devs = device::list();
            println!("Attached scanners:");
            devs.iter().for_each(|x| println!("{x}"));
        }
        cli::Action::Scan(args) => scan::run(args)?,
    }

    Ok(())
}

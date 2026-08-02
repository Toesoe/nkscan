//! Take the thumbnail pass and write it, without scanning any frames
//!
//! `cargo run --example overview -- /tmp/overview.tiff`

fn init_logging() {
    let subscriber = tracing_subscriber::fmt().with_writer(std::io::stderr);
    match tracing_subscriber::EnvFilter::try_from_default_env() {
        Ok(filter) => subscriber.with_env_filter(filter).init(),
        Err(_) => subscriber
            .with_env_filter(tracing_subscriber::EnvFilter::new("info"))
            .without_time()
            .with_target(false)
            .init(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    let out = std::env::args().nth(1).unwrap_or("overview.tiff".into());
    let device = nkscan::devices::list()
        .into_iter()
        .next()
        .ok_or("no scanner found")?;

    let mut session = nkscan::session::Session::open(&device.id)?;
    println!(
        "{} with {}",
        device.model.name(),
        session.capabilities()?.adapter_name()
    );

    let (image, dpi) = session.overview(&mut |read, total| {
        if let Some(pct) = (read * 100).checked_div(total) {
            print!("\r{pct:>3}%");
            let _ = std::io::Write::flush(&mut std::io::stdout());
        }
        nkscan::scanners::Flow::Continue
    })?;
    println!(
        "\r{} x {} at {dpi} DPI",
        image.rgb.width(),
        image.rgb.height()
    );
    image.rgb.save(&out)?;
    println!("wrote {out}");
    Ok(())
}

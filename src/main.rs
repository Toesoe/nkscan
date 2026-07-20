use nkscan::{scanners::Ls9k, scsi::linux::SgDevice};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let transport = SgDevice::open("/dev/sg4").unwrap();
    let mut scanner = Ls9k::new(transport);
    let resp = scanner.inquiry().unwrap();
    println!("{resp:#?}");
}

//! Read-only probe: identify the scanner, watch the status settle out of cold start, report the
//! loaded adapter, and dump whichever VPD pages or vendor registers you name.
//!
//! Moves no film. `RUST_LOG=debug` also shows every CDB, phase byte and raw sense.
//!
//! ```text
//! cargo run --example ls50_probe                          # the standard report
//! cargo run --example ls50_probe -- --page c1 --page f0   # plus those VPD pages
//! cargo run --example ls50_probe -- --vendor 42           # plus vendor subcode 0x42
//! ```

use clap::Parser;
use nkscan::{
    scanners::{
        FilmHolder, Scanner,
        ls50ed::{Ls50ed, PRODUCT_ID, VENDOR_ID, status::Status},
    },
    scsi::usb::UsbTransport,
};
use std::{thread::sleep, time::Duration};

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// A vital product data page to dump, in hex. Repeatable. Page 00 lists the rest.
    #[arg(long = "page", value_parser = hex_byte)]
    pages: Vec<u8>,
    /// A vendor register subcode to dump, in hex. Repeatable.
    #[arg(long = "vendor", value_parser = hex_byte)]
    vendors: Vec<u8>,
    /// How many bytes to ask each vendor register for. The firmware rejects above 13.
    #[arg(long, default_value_t = 13)]
    vendor_length: u32,
}

fn hex_byte(text: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(text.trim_start_matches("0x"), 16)
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let args = Args::parse();

    let transport = UsbTransport::open(VENDOR_ID, PRODUCT_ID)?;
    let mut scanner = Ls50ed::new(transport)?;

    let identity = scanner.identify()?;
    println!(
        "INQUIRY: vendor={:?} product={:?} revision={:?}",
        identity.vendor, identity.product, identity.revision
    );

    for attempt in 0..20 {
        let status = scanner.status()?;
        println!("STATUS[{attempt}]: {status:?}");
        if status == Status::Ready {
            break;
        }
        sleep(Duration::from_millis(500));
    }

    println!("HOLDER: {:?}", scanner.holder()?);
    println!("ADAPTER: {:?}", scanner.adapter_name());
    println!("CAPS:   {:?}", scanner.capabilities());
    println!("FRAMES: {}", scanner.sensed_frames());
    match scanner.probe_vendor(0x42, 13) {
        Ok(bytes) => println!("FEED:   {}", hex(&bytes)),
        Err(err) => println!("FEED:   {err}"),
    }

    for page in args.pages {
        match scanner.vpd_page(page) {
            Ok(bytes) => println!("PAGE {page:02X}: {}", hex(&bytes)),
            Err(err) => println!("PAGE {page:02X}: {err}"),
        }
    }

    for subcode in args.vendors {
        match scanner.probe_vendor(subcode, args.vendor_length) {
            Ok(bytes) => println!("VENDOR {subcode:02X}: {}", hex(&bytes)),
            Err(err) => println!("VENDOR {subcode:02X}: {err}"),
        }
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

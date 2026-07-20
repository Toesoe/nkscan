use crate::{scanners::Ls9k, scsi::linux::SgDevice};

mod scanners;
mod scsi;

fn main() {
    let transport = SgDevice::open("/dev/sg4").unwrap();
    let mut scanner = Ls9k::new(transport);
    let resp = scanner.inquiry().unwrap();
    println!("{resp:#?}");
}

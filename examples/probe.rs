//! Open a scanner and print what it says it can do, without moving anything
//!
//! Opening reserves the unit and writes the measurement units, and reading the capability and
//! adapter pages is a pair of inquiries. Nothing here drives the carriage or the stage, so it is
//! safe to run with film loaded.
//!
//! `cargo run --example probe`

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let found = nkscan::devices::list();
    if found.is_empty() {
        println!("No scanners found.");
        return Ok(());
    }

    for device in &found {
        println!("{}  {}", device.id, device.model.name());
        println!("  driven:      {}", device.model.is_driven());
        println!("  family:      {:?}", device.model.family());
        println!("  interface:   {:?}", device.model.interface());
        if !device.model.is_driven() {
            println!("  (no driver, so it cannot be opened)");
            continue;
        }

        let mut session = nkscan::session::Session::open(&device.id)?;
        let caps = session.capabilities()?;
        println!("  adapter:     {} ({})", caps.adapter_name(), caps.adapter);
        println!("  optical dpi: {}", caps.resolution.optical);
        println!("  dpi ladder:  {:?}", caps.resolution.ladder);
        println!(
            "  depth:       {} native, offers {:?}",
            caps.depth.native, caps.depth.offered
        );
        println!("  multisample: {:?}", caps.multisample);
        println!("  single line: {}", caps.single_line);
        println!("  focus range: {:?}", caps.focus_range);
        println!("  exposure:    {:?}", caps.exposure);
        println!("  eject:       {:?}", caps.eject);
        println!("  overview:    {}", caps.overview);
        println!("  frames:      {:?}", caps.frames);
        println!("  batch:       {}", caps.batch);
        println!("  strip offset:{}", caps.strip_offset);
        println!("  max area mm: {:?}", caps.max_area_mm);
        println!("  media in:    {}", session.media_loaded()?);
        drop(session);

        // The three signals page 0xC8 carries, which the shared Adapter vocabulary narrows away.
        // A class does not pin a part number, so this is what a capture per holder has to record.
        if device.model.family() == nkscan::model::Family::MediumFormat {
            let mut scanner = nkscan::scanners::ls9000::Ls9000::new(device.attach.open()?)?;
            let reading = scanner.holder_reading()?;
            println!("  --- page 0xC8, raw ---");
            println!("  class:       {:?}", reading.class);
            println!("  byte 4:      {} (meaning unestablished)", reading.byte_4);
            println!(
                "  width:       {} dots ({:.2} mm at {} dpi)",
                reading.width_dots,
                nkscan::capability::dots_to_mm(
                    u32::from(reading.width_dots),
                    caps.resolution.optical
                ),
                caps.resolution.optical
            );
        }
    }
    Ok(())
}

use clap::{Parser, Subcommand};
use nkscan::{protocol::caps::film::FilmFormat, scan::profile::Film};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about)]
/// Scan film on a Nikon Coolscan
pub struct Cli {
    #[command(subcommand)]
    pub action: Action,
}

#[derive(Subcommand)]
pub enum Action {
    /// List available scanners
    List,
    /// Perform a scan. Defaults to batch scanning with sensible defaults.
    Scan {
        /// The scanner to connect to. Optional, will default to the first found.
        device: Option<String>,

        /// Where to write, as a path prefix. Each frame becomes <basename>_<n>.tiff, and its infrared mask <basename>_<n>_IR.tiff
        #[arg(long, default_value = "scan")]
        basename: PathBuf,

        /// Autoexpose per channel. Better dynamic range, but no longer "calibrated".
        #[arg(long)]
        unlock_wb: bool,

        /// Autoexpose the first frame and reuse that exposure across all frames.
        #[arg(long)]
        lock_ae: bool,

        /// Resolution. Defaults to scanner maximum.
        #[arg(long)]
        dpi: Option<u16>,

        /// Number of samples. Defaults to 1.
        #[arg(long, default_value_t = 1)]
        samples: u8,

        /// Singleline CCD mode. Only supported on multiline CCD scanners.
        #[arg(long)]
        superfine: bool,

        /// Which frame(s) to scan, comma separated. Defaults to all detected.
        /// Naming any stops after one holder rather than batching.
        #[arg(long, value_delimiter = ',')]
        frames: Vec<usize>,

        /// Include the IR pass
        #[arg(long)]
        ir: bool,

        /// Don't eject at the end of the strip
        #[arg(long)]
        no_eject: bool,

        /// Film format. One of: 135, 16, 645, 66, 67, 68, 69, or a custom frame
        /// height in mm. Defaults to whatever the loaded holder fixes.
        #[arg(long, value_parser = parse_format)]
        format: Option<FilmFormat>,

        /// Film type, which picks the color profile the scans are tagged with
        #[arg(long, value_enum, default_value_t = FilmType::Negative)]
        film: FilmType,
    },
}

/// The film types Nikon profiled, as flag values
#[derive(Copy, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum FilmType {
    /// Slide film
    Positive,
    /// Color negative
    Negative,
    /// Kodachrome, whose dyes need their own profile
    Kodachrome,
    /// Black and white negative
    Mono,
}

impl From<FilmType> for Film {
    fn from(f: FilmType) -> Self {
        match f {
            FilmType::Positive => Film::Positive,
            FilmType::Negative => Film::Negative,
            FilmType::Kodachrome => Film::Kodachrome,
            FilmType::Mono => Film::MonochromeNegative,
        }
    }
}

/// A film format flag, by name or as a frame height in millimetres
///
/// The named ones are what the holders take; anything else is a height, which
/// is what a format nobody named still needs
pub fn parse_format(flag: &str) -> Result<FilmFormat, String> {
    Ok(match flag {
        "135" => FilmFormat::F135,
        "16" => FilmFormat::F16,
        "645" => FilmFormat::F645,
        "66" => FilmFormat::F66,
        "67" => FilmFormat::F67,
        "68" => FilmFormat::F68,
        "69" => FilmFormat::F69,
        mm => FilmFormat::Custom(
            mm.parse()
                .map_err(|_| format!("'{mm}' is neither a film format nor a height in mm"))?,
        ),
    })
}

/// What `parse_format` would take for this format, for saying what is on offer
pub fn format_name(format: &FilmFormat) -> String {
    match format {
        FilmFormat::F135 => "135".into(),
        FilmFormat::F16 => "16".into(),
        FilmFormat::F645 => "645".into(),
        FilmFormat::F66 => "66".into(),
        FilmFormat::F67 => "67".into(),
        FilmFormat::F68 => "68".into(),
        FilmFormat::F69 => "69".into(),
        FilmFormat::Custom(mm) => mm.to_string(),
    }
}

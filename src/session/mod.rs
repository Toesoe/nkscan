//! Driving a scanner through a whole strip, whichever model is on the other end
//!
//! The layer above the per-model drivers: it wakes the mechanism, works out where the frames
//! are, settles the exposure and scans them. A [`Session`] is one exclusive hold on one device,
//! opened once and used for as many frames as the film has.
//!
//! Everything here is model-agnostic. What varies per model lives behind [`Driver`], and what a
//! particular unit reports lives in [`DeviceLimits`](crate::scanners::nikon::limits).

use crate::capability::resolve::{ResolvedFrame, ResolvedPrepare};
use crate::capability::unsupported::{Allowed, Feature, Unsupported};
use crate::capability::{Capabilities, EjectAction};
use crate::decode::Image;
use crate::devices::{DeviceInfo, Model};
use crate::model::Protocol;
use crate::scanners::{ProgressFn, nikon::ChannelExposures};
use crate::scsi;
use std::io;
use std::time::Duration;

/// How often the media wait asks again
const MEDIA_POLL: Duration = Duration::from_millis(500);

mod ls50;
mod ls5000;
mod ls9000;

/// Where to put the focus motor before a pass
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusMode {
    /// Let the scanner find it, once per frame
    Auto,
    /// Drive the motor to this setpoint and leave it there
    At(u16),
}

impl std::str::FromStr for FocusMode {
    type Err = String;

    fn from_str(text: &str) -> std::result::Result<Self, String> {
        match text {
            "auto" => Ok(FocusMode::Auto),
            other => other
                .parse()
                .map(FocusMode::At)
                .map_err(|_| format!("expected `auto` or a setpoint, got `{other}`")),
        }
    }
}

/// How the frames on the loaded film get located
///
/// Millimeter figures become whole dots at the model's own pitch, and a frame length then floors
/// to whatever the model's interleave needs, so a placement given in millimeters is not exactly
/// recoverable from the frames it produces.
#[derive(Debug, Clone, PartialEq)]
pub enum Placement {
    /// Look for them: a low-resolution overview pass, then find the frames in it
    ///
    /// Only where [`Overview::Available`](crate::capability::Overview) is reported.
    Detect { frames: usize },
    /// Place them arithmetically along the feed
    ///
    /// `frames` unset asks the scanner how many it can see. `pitch_mm` unset uses what it
    /// reports for the loaded holder. `offsets_mm` shifts each frame along the feed, the last
    /// value repeating, so one value shifts the whole strip.
    Pitch {
        frames: Option<u32>,
        pitch_mm: Option<f32>,
        offsets_mm: Vec<f32>,
    },
    /// Take the frame table the transport itself reports, where it senses one
    ///
    /// Falls back to an even pitch on an adapter that reports none.
    Sensed { frames: Option<u32> },
}

/// How the gain for a pass is decided
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exposure {
    /// Meter the film and use what that measures
    Auto { lock_white_balance: bool },
    /// Hold these gains, skipping metering entirely
    ///
    /// `ir` unset leaves the infrared gain to the model, which is not the same thing as zero:
    /// one model drives its infrared off a zeroed field and another meters it.
    Fixed { visible: [u32; 3], ir: Option<u32> },
}

/// What a session needs before it can place frames
#[derive(Debug, Clone, PartialEq)]
pub struct Prepare {
    pub placement: Placement,
    pub exposure: Exposure,
    /// How long to wait for film to be loaded. Zero refuses an empty scanner rather than waiting.
    pub wait_for_media: Duration,
}

/// One frame's pass
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameSettings {
    pub dpi: u16,
    pub ir: bool,
    pub focus: FocusMode,
    pub multisample: u8,
    /// The slower single-line CCD readout, where the model has one
    pub single_line: bool,
    /// A sub-rectangle of the placed frame, as fractions of it: (x0, y0, x1, y1)
    ///
    /// Not implemented. No capture crops a frame, and the alignment a window has to keep is
    /// per model, so this is refused rather than guessed at.
    pub window: Option<(f32, f32, f32, f32)>,
}

impl Default for FrameSettings {
    fn default() -> Self {
        Self {
            dpi: 4000,
            ir: false,
            focus: FocusMode::Auto,
            multisample: 1,
            single_line: false,
            window: None,
        }
    }
}

/// Anything a session can fail with
///
/// The one error a consumer sees, since a session is the only way in. The layers below keep
/// their own: this classifies rather than replaces them.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no scanner at {0}")]
    NotFound(String),
    #[error("{0} does not name a scanner: expected <model>@<location>")]
    BadDeviceId(String),
    #[error("could not reach the scanner: {0}")]
    Transport(#[from] io::Error),
    #[error(transparent)]
    Scsi(#[from] scsi::Error),
    #[error("could not decode the scan: {0}")]
    Decode(String),
    /// No film, no holder, or nothing recognizable on the strip
    #[error("{0}")]
    Media(String),
    #[error(transparent)]
    Unsupported(#[from] Unsupported),
    #[error("the pass was cancelled")]
    Cancelled,
}

/// Any model's streamed pass, classified
///
/// Generic over the decode error so all three models' `ScanError` aliases convert the same way.
impl<E: std::fmt::Display> From<crate::scanners::ReadError<E>> for Error {
    fn from(error: crate::scanners::ReadError<E>) -> Self {
        use crate::scanners::ReadError;
        match error {
            ReadError::Scsi(error) => Error::Scsi(error),
            ReadError::Decode(error) => Error::Decode(error.to_string()),
            ReadError::Cancelled => Error::Cancelled,
        }
    }
}

/// Refuse a unit that is not the model the id claimed
///
/// An sg node is assigned in probe order, so an id from before a replug can point at something
/// else entirely. Cheap to check, and the alternative is driving the wrong scanner.
pub(crate) fn confirm_model(
    identity: crate::scsi::cdbs::InquiryResponse,
    model: Model,
) -> Result<(), Error> {
    let product = identity.product.trim();
    if !product.eq_ignore_ascii_case(model.name()) {
        return Err(Error::NotFound(format!(
            "{product} answered where a {} was expected",
            model.name()
        )));
    }
    tracing::info!("Connected to {} {product}", identity.vendor.trim());
    Ok(())
}

/// The table for this pairing, refined by what this unit reports
///
/// The driver supplies its own resolution rungs, since which divisions exist is per model and
/// stays with the driver that encodes them.
pub(crate) fn reported_capabilities<D: Copy>(
    model: Model,
    adapter: crate::adapter::Adapter,
    reported: crate::scanners::nikon::limits::DeviceLimits,
    ladder: &[D],
    to_dpi: fn(D) -> u16,
) -> Capabilities {
    let rungs: Vec<u16> = ladder.iter().copied().map(to_dpi).collect();
    crate::capability::table::compute(model, adapter).refine(&reported, &rungs)
}

/// One model's half of a [`Session`]
///
/// Object-safe on purpose: the model is chosen at runtime and nothing above this names it, which
/// is what lets a caller reach a session without naming a transport either.
pub(crate) trait Driver: Send {
    fn capabilities(&mut self) -> Result<Capabilities, Error>;
    fn media_loaded(&mut self) -> Result<bool, Error>;
    fn prepare(
        &mut self,
        prepare: &ResolvedPrepare,
        progress: &mut ProgressFn<'_>,
    ) -> Result<usize, Error>;
    fn frames(&self) -> usize;
    /// How many frames the loaded holder reports, where the model can be asked
    fn sensed_frames(&mut self) -> Option<u32> {
        None
    }
    fn scan_frame(
        &mut self,
        index: usize,
        settings: &ResolvedFrame,
        progress: &mut ProgressFn<'_>,
    ) -> Result<Image, Error>;
    fn lock_gain(&mut self);
    fn gain(&self) -> ChannelExposures;
    /// The low-resolution pass, where the model drives one
    ///
    /// Defaults to refusing: the pass exists in hardware on most adapters, but writing it is per
    /// model and only one driver has.
    fn overview(&mut self, _progress: &mut ProgressFn<'_>) -> Result<(Image, u16), Error> {
        Err(Unsupported::not_implemented(
            Feature::Overview,
            "no overview pass is written for this model",
        )
        .into())
    }

    /// Settle whatever loading the film raised, once it is known to be in
    ///
    /// Split out of the wait so the polling half can live above the drivers. A model with
    /// nothing to settle keeps the default.
    fn after_media_ready(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn eject(&mut self) -> Result<(), Error>;
    fn abort(&mut self) -> Result<(), Error>;
}

/// An exclusive hold on one scanner
pub struct Session {
    device: DeviceInfo,
    driver: Box<dyn Driver>,
}

impl Session {
    /// Claim the scanner `id` names, as [`devices::list`] reported it
    ///
    /// The unit is asked who it is again on the way up, so an id left over from before something
    /// was replugged fails here rather than driving the wrong scanner.
    pub fn open(id: &str) -> Result<Self, Error> {
        let (model, attach) =
            DeviceInfo::parse_id(id).ok_or_else(|| Error::BadDeviceId(id.into()))?;
        let transport = attach.open()?;
        Self::with_transport(
            DeviceInfo {
                id: id.to_owned(),
                model,
                attach,
            },
            transport,
        )
    }

    /// Build a session over a transport the caller already has
    ///
    /// The way in for a test, and for anything that supplies its own transport rather than
    /// letting enumeration find one.
    pub(crate) fn with_transport(
        device: DeviceInfo,
        transport: Box<dyn crate::scsi::Transport + Send>,
    ) -> Result<Self, Error> {
        let model = device.model;
        // Dispatched on the dialect rather than on the model, so giving one of the recognized
        // models a driver is a line in `Model::protocol` and nothing here
        let driver: Box<dyn Driver> = match model.protocol() {
            Some(Protocol::Ls9000) => Box::new(ls9000::Ls9000Driver::open(transport, model)?),
            Some(Protocol::Ls50) => Box::new(ls50::Ls50Driver::open(transport, model)?),
            Some(Protocol::Ls5000) => Box::new(ls5000::Ls5000Driver::open(transport, model)?),
            None => {
                return Err(Unsupported::not_implemented(
                    Feature::Model,
                    "crate::model::Model::protocol, and the stub module for this model",
                )
                .into());
            }
        };
        Ok(Self { device, driver })
    }

    pub fn device(&self) -> &DeviceInfo {
        &self.device
    }

    /// What this unit reports, which refines the static table
    /// [`Model::capabilities`](crate::devices::Model::capabilities) has to answer from
    pub fn capabilities(&mut self) -> Result<Capabilities, Error> {
        self.driver.capabilities()
    }

    /// Refuse settings this model does not have, before anything mechanical happens
    ///
    /// Worth calling first: a scan discovers these only once it is building the pass, which on
    /// some models is after a focus and a metering run have already taken a minute.
    pub fn check(&mut self, prepare: &Prepare, settings: &FrameSettings) -> Result<(), Error> {
        self.resolve(prepare, settings).map(|_| ())
    }

    /// Check both halves against this unit and hand back what a pass will actually run with
    ///
    /// Not something a caller has to remember: [`prepare`](Self::prepare) and
    /// [`scan_frame`](Self::scan_frame) resolve too, and a driver is only reachable through a
    /// resolved value. Worth calling first anyway, since a scan otherwise discovers a refusal
    /// after a focus and a metering run have already spent a minute.
    pub fn resolve(
        &mut self,
        prepare: &Prepare,
        settings: &FrameSettings,
    ) -> Result<(ResolvedPrepare, ResolvedFrame), Error> {
        let capabilities = self.capabilities()?;
        Ok((
            capabilities.resolve_prepare(prepare)?,
            capabilities.resolve_frame(settings)?,
        ))
    }

    /// Whether there is film in the scanner now
    pub fn media_loaded(&mut self) -> Result<bool, Error> {
        self.driver.media_loaded()
    }

    /// Wake the mechanism, settle the gain and place the frames, returning how many were placed
    pub fn prepare(
        &mut self,
        prepare: &Prepare,
        progress: &mut ProgressFn<'_>,
    ) -> Result<usize, Error> {
        let resolved = self.capabilities()?.resolve_prepare(prepare)?;
        // Waiting for film is model-agnostic, so it happens once here rather than in each
        // driver. Two of the three used to take the field and ignore it.
        self.wait_for_media(resolved.wait_for_media(), progress)?;
        self.driver.after_media_ready()?;
        self.driver.prepare(&resolved, progress)
    }

    /// Poll until there is film in the scanner, or until `timeout` runs out
    ///
    /// Zero refuses an empty scanner rather than waiting, which is what a batch run wants between
    /// strips and what a one-shot run wants immediately.
    fn wait_for_media(
        &mut self,
        timeout: Duration,
        progress: &mut ProgressFn<'_>,
    ) -> Result<(), Error> {
        let deadline = std::time::Instant::now() + timeout;
        loop {
            if self.driver.media_loaded()? {
                return Ok(());
            }
            if std::time::Instant::now() >= deadline {
                return Err(Error::Media("no film is loaded".into()));
            }
            if progress(0, 0) == crate::scanners::Flow::Cancel {
                return Err(Error::Cancelled);
            }
            std::thread::sleep(MEDIA_POLL);
        }
    }

    /// How many frames [`prepare`](Self::prepare) placed
    pub fn frames(&self) -> usize {
        self.driver.frames()
    }

    /// How many frames the loaded holder reports, `None` where the model cannot be asked
    ///
    /// A cheap vendor page read, so this answers before anything mechanical happens — which is
    /// also the only time it is worth trusting on the models that clear it as the film moves.
    /// `Some(0)` is an empty transport, and is not the same answer as `None`.
    pub fn sensed_frames(&mut self) -> Option<u32> {
        self.driver.sensed_frames()
    }

    /// Focus, expose and scan one of the placed frames
    pub fn scan_frame(
        &mut self,
        index: usize,
        settings: &FrameSettings,
        progress: &mut ProgressFn<'_>,
    ) -> Result<Image, Error> {
        if index >= self.driver.frames() {
            let placed = self.driver.frames() as u32;
            return Err(Unsupported::out_of_range(
                Feature::FramePlacement,
                index as u32,
                Allowed::Range {
                    min: 0,
                    max: placed.saturating_sub(1),
                },
            )
            .into());
        }
        // Every path into a driver resolves, so a setting the capability table refuses cannot
        // reach one whether or not a caller thought to check first
        let resolved = self.capabilities()?.resolve_frame(settings)?;
        self.driver.scan_frame(index, &resolved, progress)
    }

    /// The low-resolution pass Nikon Scan calls a thumbnail
    ///
    /// The whole strip in one image, which is what a host needs in order to show the film and
    /// let someone place frames on it. [`prepare`](Self::prepare) with [`Placement::Detect`]
    /// takes the same pass and finds the frames itself; this hands the raster back instead.
    ///
    /// Returns the resolution it ran at alongside the image, since mapping a point on the
    /// thumbnail back to a position on the film is the reason to have it.
    pub fn overview(&mut self, progress: &mut ProgressFn<'_>) -> Result<(Image, u16), Error> {
        if !self.capabilities()?.overview {
            return Err(Unsupported::not_present(Feature::Overview, &self.capabilities()?).into());
        }
        self.driver.overview(progress)
    }

    /// Hold whatever gain the last scan settled on, so the rest of the roll matches it
    pub fn lock_gain(&mut self) {
        self.driver.lock_gain();
    }

    /// The gain the next pass will use
    pub fn gain(&self) -> ChannelExposures {
        self.driver.gain()
    }

    /// Send the film back out
    pub fn eject(&mut self) -> Result<EjectAction, Error> {
        let action = self.capabilities()?.eject;
        if action == EjectAction::Unavailable {
            // Refused before the bus is touched: a mount adapter has nothing to give back, and
            // the command would either fail or move something that should not move
            return Err(Unsupported::not_present(Feature::Eject, &self.capabilities()?).into());
        }
        self.driver.eject()?;
        Ok(action)
    }

    /// Throw away a pass nobody is going to read, so the handle stays usable
    pub fn abort(&mut self) -> Result<(), Error> {
        self.driver.abort()
    }
}

/// Resolve a requested resolution against a model's ladder and the range the device reports
///
/// Dividing the sensor evenly is not enough: a ladder entry under the reported floor is refused
/// by the firmware, so only the offered ones are named back.
pub fn resolve_dpi<D: Copy>(
    requested: u16,
    ladder: &[D],
    offered: crate::scanners::nikon::limits::ResolutionRange,
    to_dpi: fn(D) -> u16,
) -> Result<D, Error> {
    let legal: Vec<D> = ladder
        .iter()
        .copied()
        .filter(|&mode| offered.allows(to_dpi(mode)))
        .collect();
    if let Some(&mode) = legal.iter().find(|&&mode| to_dpi(mode) == requested) {
        return Ok(mode);
    }

    Err(Unsupported::not_one_of(
        Feature::Resolution,
        u32::from(requested),
        legal.iter().map(|&mode| u32::from(to_dpi(mode))),
    )
    .into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devices::Attach;
    use crate::scanners::Flow;
    use crate::scanners::ls9000::{capabilities as ls9000_capabilities, geometry::Dpi};
    use crate::scanners::nikon::limits::ResolutionRange;
    use crate::scsi::mock::MockTransport;
    use std::path::PathBuf;

    /// A transport that answers everything opening an LS-9000 session asks
    fn mock_ls9000() -> MockTransport {
        // Page 0xC8 as a real capture answers it with strip film in
        let holder = [
            0x01, 0x00, 0x00, 0x02, 0x06, 0x00, 0x00, 0x08, 0xbc, 0x00, 0x00, 0x23, 0x04, 0x00,
            0x00, 0x00, 0x00,
        ];
        let mut raw = vec![0x06, 0xC8];
        raw.extend_from_slice(&(holder.len() as u16).to_be_bytes());
        raw.extend_from_slice(&holder);

        MockTransport::new()
            .with_identity("Nikon", "LS-9000 ED")
            .with_page(0xC1, ls9000_capabilities::fixture::raw_page())
            .with_page(0xC8, raw)
    }

    fn ls9000_session(transport: MockTransport) -> Session {
        Session::with_transport(
            DeviceInfo {
                id: "ls9000@/dev/sg0".into(),
                model: Model::Ls9000,
                attach: Attach::Scsi {
                    path: PathBuf::from("/dev/sg0"),
                },
            },
            Box::new(transport),
        )
        .expect("opens")
    }

    /// The command order a scan through a session has to keep
    ///
    /// The same sequence `ls9000::tests::the_workflow_issues_commands_in_the_order_the_scanner_needs`
    /// pins against the driver directly. Both exist because this one is what proves the workflow
    /// moving up here did not reorder any of it.
    #[test]
    fn a_session_scan_keeps_the_order_the_scanner_needs() {
        let transport = mock_ls9000();
        let mut session = ls9000_session(transport.clone());

        let prepare = Prepare {
            // 36 dots is the shortest window a three-line pass divides into
            placement: Placement::Pitch {
                frames: Some(1),
                pitch_mm: Some(36.0 * 25.4 / 4000.0),
                offsets_mm: Vec::new(),
            },
            exposure: Exposure::Auto {
                lock_white_balance: false,
            },
            wait_for_media: Duration::from_secs(1),
        };
        let settings = FrameSettings {
            dpi: 666,
            ..FrameSettings::default()
        };

        session.check(&prepare, &settings).expect("supported");
        let placed = session
            .prepare(&prepare, &mut |_, _| Flow::Continue)
            .expect("prepares");
        assert_eq!(placed, 1);
        session
            .scan_frame(0, &settings, &mut |_, _| Flow::Continue)
            .expect("scans");

        assert_eq!(
            transport.opcode_sequence(),
            [
                // --- opening the handle
                0x00, // TEST UNIT READY, waiting out power-on and draining unit attentions
                0x12, // INQUIRY, the capability page then the identity
                0x16, // RESERVE
                0xC0, // vendor ABORT, clearing a pass left by a killed process
                0x15, // MODE SELECT, the measurement units
                0x12, // INQUIRY, confirming the model is the one the id claimed
                // --- the holder, then letting the mechanism settle
                0x00, // TEST UNIT READY
                // --- calibrate, the session preamble
                0x28, // READ, the frame setup per channel
                0x24, // SET WINDOW per channel
                0xE1, // vendor read, the staged focus
                0xE0, // vendor write, committing it
                0xC1, // vendor TRIGGER
                0x28, // READ, the current frame table
                0x2A, // SEND, the nominal one
                0x28, // READ, the per-channel calibration
                // --- the frame table this strip actually has
                0x2A, // SEND, the boundaries
                // --- scanning a frame, which resolves its settings before anything moves
                0x12, // INQUIRY, the adapter page: what a frame may ask for depends on it
                // --- autofocus, which needs the table written first
                0xE0, // vendor write, the focus point
                0xC1, // vendor TRIGGER
                0x00, // TEST UNIT READY, waiting out the pass
                0xE1, // vendor read, where it landed
                // --- metering, two passes by default
                0x24, // SET WINDOW per channel
                0x1B, // SCAN
                0x00, // TEST UNIT READY
                0x28, // READ, the image
                0x24, // SET WINDOW, the second metering pass
                0x1B, // SCAN
                0x00, // TEST UNIT READY
                0x28, // READ
                // --- the scan
                0x24, // SET WINDOW per channel
                0x1B, // SCAN
                0x00, // TEST UNIT READY
                0x28, // READ, the image
            ]
        );
    }

    /// Metering starts from the model's own default gain, not from what the last frame settled
    /// on, since scaling a gain that is already scaled compounds down the roll
    #[test]
    fn metering_starts_from_the_default_gain() {
        let transport = mock_ls9000();
        let mut session = ls9000_session(transport.clone());
        let prepare = Prepare {
            placement: Placement::Pitch {
                frames: Some(1),
                pitch_mm: Some(36.0 * 25.4 / 4000.0),
                offsets_mm: Vec::new(),
            },
            exposure: Exposure::Auto {
                lock_white_balance: false,
            },
            wait_for_media: Duration::from_secs(1),
        };
        let settings = FrameSettings {
            dpi: 666,
            ..FrameSettings::default()
        };
        session
            .prepare(&prepare, &mut |_, _| Flow::Continue)
            .expect("prepares");
        session
            .scan_frame(0, &settings, &mut |_, _| Flow::Continue)
            .expect("scans");

        // A window descriptor is 8 header bytes then 50 of descriptor, whose vendor tail carries
        // the gain as a u32 in its last four bytes
        let red = crate::scanners::ls9000::calibration::DEFAULT_GAIN.red;
        let staged: Vec<u32> = transport
            .data_outs(0x24)
            .iter()
            .filter(|out| out.len() == 58)
            .map(|out| u32::from_be_bytes([out[54], out[55], out[56], out[57]]))
            .collect();
        assert!(
            staged.contains(&red),
            "no window staged the default red gain {red}, saw {staged:?}"
        );
    }

    fn offered(min: u16) -> ResolutionRange {
        ResolutionRange {
            optical: 4000,
            min,
            max: 4000,
        }
    }

    #[test]
    fn a_resolution_the_device_reaches_resolves() {
        let mode = resolve_dpi(1333, &Dpi::ALL, offered(666), Dpi::to_dpi).expect("offered");
        assert_eq!(mode.to_dpi(), 1333);
        // the same rung on a unit that divides further
        let mode = resolve_dpi(333, &Dpi::ALL, offered(90), Dpi::to_dpi).expect("offered");
        assert_eq!(mode.to_dpi(), 333);
    }

    /// Under the floor and off the ladder are the same complaint, and neither offers a rung the
    /// device will not take
    ///
    /// Asserted on the refusal itself rather than on its wording: a caller picking another
    /// resolution reads `allowed`, and prose it had to parse is exactly what this replaced.
    #[test]
    fn a_resolution_the_device_does_not_reach_is_refused() {
        use crate::capability::unsupported::{Allowed, Feature, Reason};
        for asked in [333, 800] {
            let error =
                resolve_dpi(asked, &Dpi::ALL, offered(666), Dpi::to_dpi).expect_err("refused");
            let Error::Unsupported(refusal) = error else {
                panic!("a resolution off the ladder is an unsupported request");
            };
            assert_eq!(refusal.feature, Feature::Resolution);
            assert_eq!(
                refusal.reason,
                Reason::OutOfRange {
                    asked: u32::from(asked),
                    allowed: Allowed::Values(vec![4000, 2000, 1333, 666]),
                }
            );
        }
    }

    /// A session has to cross a thread, which is what a caller running a scan on a worker needs
    #[test]
    fn a_session_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Session>();
    }

    #[test]
    fn an_id_that_names_nothing_is_refused_before_anything_opens() {
        let error = Session::open("nonsense").err().expect("refused");
        assert!(matches!(error, Error::BadDeviceId(_)), "{error:?}");
    }
}

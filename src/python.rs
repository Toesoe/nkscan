//! The Python extension module
//!
//! A thin skin over [`session`](crate::session): it converts arguments, hands the decoded planes
//! to numpy without copying them, and releases the interpreter for the minutes a pass takes.
//!
//! Nothing here knows about any particular Python consumer. The types are shaped so an adapter
//! for one is a translation rather than a rewrite.

use crate::capability::{self, Capabilities};
use crate::decode::Image;
use crate::devices::{self, DeviceInfo};
use crate::scanners::Flow;
use crate::session::{Error, Exposure, FocusMode, FrameSettings, Placement, Prepare, Session};
use numpy::{IntoPyArray, PyArray2, PyArray3, PyArrayMethods};
use pyo3::exceptions::{PyPermissionError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3_stub_gen::{create_exception, define_stub_info_gatherer, derive::*};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// How often a pass calls back into Python
///
/// A full-resolution pass is tens of thousands of chunks and re-acquiring the interpreter for
/// every one buys nothing a caller can use. This is also how long a cancellation takes to notice.
const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);

create_exception!(nkscan, ScannerError, PyRuntimeError, "Any scanner failure.");
create_exception!(
    nkscan,
    TransientError,
    ScannerError,
    "A failure worth retrying: a link glitch, a busy device, a short read."
);
create_exception!(nkscan, TransportError, TransientError, "The link faltered.");
create_exception!(
    nkscan,
    DeviceBusy,
    TransientError,
    "Something else holds it."
);
create_exception!(nkscan, DeviceNotFound, ScannerError, "No such scanner.");
create_exception!(
    nkscan,
    MediaError,
    ScannerError,
    "No film, or nothing recognizable on it. Retrying will not help."
);
create_exception!(
    nkscan,
    UnsupportedError,
    ScannerError,
    "This scanner does not have that setting."
);
create_exception!(nkscan, ScanCancelled, ScannerError, "The pass was stopped.");

/// Turn a session failure into the exception a caller can act on
///
/// The retryable ones share [`TransientError`] as a base so a consumer writes one `except` and
/// which failures are worth retrying stays a decision made here.
fn to_py(error: Error) -> PyErr {
    use crate::scsi::Error as Scsi;
    match error {
        Error::NotFound(m) | Error::BadDeviceId(m) => DeviceNotFound::new_err(m),
        Error::Media(m) => MediaError::new_err(m),
        Error::Unsupported(refusal) => unsupported_err(refusal),
        Error::Cancelled => ScanCancelled::new_err("the pass was cancelled"),
        // A short or scrambled stream is exactly what a retry fixes
        Error::Decode(m) => TransientError::new_err(m),
        Error::Transport(io) => match io.kind() {
            std::io::ErrorKind::PermissionDenied => PyPermissionError::new_err(format!(
                "{io}. A scanner device needs read and write access: a udev rule on Linux, or an \
                 elevated prompt on Windows."
            )),
            std::io::ErrorKind::NotFound => DeviceNotFound::new_err(io.to_string()),
            _ => TransportError::new_err(io.to_string()),
        },
        Error::Scsi(scsi) => match &scsi {
            Scsi::Transport(_) | Scsi::HostAdapter { .. } | Scsi::InvalidResponse(_) => {
                TransportError::new_err(scsi.to_string())
            }
            // Reissuing is what clears these
            Scsi::Status { sense: Some(s), .. } if s.key == 0x02 || s.key == 0x06 => {
                TransportError::new_err(scsi.to_string())
            }
            Scsi::Unsupported(_) => UnsupportedError::new_err(scsi.to_string()),
            Scsi::Status { .. } => ScannerError::new_err(scsi.to_string()),
        },
    }
}

/// A refusal, carrying enough for a caller to branch instead of matching on the wording
///
/// The message is still the message. Alongside it the exception gets `feature`, `reason` and,
/// where the refusal was about a value, `allowed` — so a consumer can grey out a control, pick
/// another resolution, or tell "this scanner cannot" apart from "this library does not yet".
fn unsupported_err(refusal: capability::unsupported::Unsupported) -> PyErr {
    use capability::unsupported::{Allowed, Reason};
    let error = UnsupportedError::new_err(refusal.to_string());
    Python::attach(|py| {
        let value = error.value(py);
        let _ = value.setattr("feature", refusal.feature.slug());
        let _ = value.setattr("reason", refusal.reason.slug());
        let allowed = match &refusal.reason {
            Reason::OutOfRange {
                allowed: Allowed::Values(values),
                ..
            } => values
                .clone()
                .into_pyobject(py)
                .ok()
                .map(|v| v.into_any().unbind()),
            Reason::OutOfRange {
                allowed: Allowed::Range { min, max },
                ..
            } => (*min, *max)
                .into_pyobject(py)
                .ok()
                .map(|v| v.into_any().unbind()),
            _ => None,
        };
        let _ = value.setattr("allowed", allowed);
        let _ = value.setattr(
            "asked",
            match &refusal.reason {
                Reason::OutOfRange { asked, .. } => Some(*asked),
                _ => None,
            },
        );
    });
    error
}

/// What a scanner can do, for the model and the adapter it has loaded
///
/// A flat view of `nkscan::capability::Capabilities`. The Rust side carries richer enums; this
/// projects them into plain values, since that is what reads well from Python.
#[gen_stub_pyclass]
#[pyclass(name = "Capabilities", frozen, get_all, module = "nkscan")]
struct PyCapabilities {
    model: String,
    /// The loaded adapter's part number, or a description where the part is not pinned down
    adapter: String,
    dpi: Vec<u32>,
    /// The sensor's own pitch, which is not 4000 on every model
    optical_dpi: u32,
    depths: Vec<u32>,
    multisample: Vec<u32>,
    ir_channel: bool,
    kodachrome_ice: bool,
    max_area_mm: (f32, f32),
    /// Whether the host meters, which is also what makes a white balance lock possible
    auto_exposure: bool,
    lock_white_balance: bool,
    single_line: bool,
    /// What ejecting does here: `none`, `holder`, `film`, `rewind` or `feed_next`
    eject: String,
    can_eject: bool,
    /// Whether this adapter has a thumbnail pass
    overview: bool,
    /// How many frames the adapter presents, where that is a fixed property of it
    frames: Option<u32>,
    /// Whether the frames have to be found rather than being mechanically fixed
    detects_frames: bool,
    /// Whether the transport senses the frames and reports a table
    senses_frames: bool,
    batch: bool,
    strip_offset: bool,
    focus_range: Option<(u16, u16)>,
}

impl From<Capabilities> for PyCapabilities {
    fn from(c: Capabilities) -> Self {
        use capability::{EjectAction, ExposureControl, FrameLocation};
        Self {
            model: c.model.name().to_owned(),
            adapter: c.adapter_name(),
            dpi: c.resolution.ladder.iter().map(|&d| u32::from(d)).collect(),
            optical_dpi: u32::from(c.resolution.optical),
            depths: c.depth.offered.iter().map(|&d| u32::from(d)).collect(),
            multisample: c.multisample.iter().map(|&n| u32::from(n)).collect(),
            ir_channel: c.ice.infrared,
            kodachrome_ice: c.ice.kodachrome,
            max_area_mm: c.max_area_mm,
            auto_exposure: matches!(c.exposure, ExposureControl::Host { .. }),
            lock_white_balance: matches!(
                c.exposure,
                ExposureControl::Host {
                    lock_white_balance: true
                }
            ),
            single_line: c.single_line,
            eject: match c.eject {
                EjectAction::Unavailable => "none",
                EjectAction::EjectHolder => "holder",
                EjectAction::EjectFilm => "film",
                EjectAction::RewindFilm => "rewind",
                EjectAction::FeedNextSlide => "feed_next",
            }
            .to_owned(),
            can_eject: c.eject != EjectAction::Unavailable,
            overview: c.overview,
            frames: match c.frames {
                FrameLocation::Mechanical(n) => Some(u32::from(n)),
                _ => None,
            },
            detects_frames: matches!(c.frames, FrameLocation::Detected),
            senses_frames: matches!(c.frames, FrameLocation::Reported),
            batch: c.batch,
            strip_offset: c.strip_offset,
            focus_range: c.focus_range,
        }
    }
}

/// A scanner the search found, not yet opened
#[gen_stub_pyclass]
#[pyclass(name = "Device", frozen, get_all, module = "nkscan")]
struct PyDevice {
    id: String,
    vendor: String,
    model: String,
    capabilities: Py<PyCapabilities>,
}

/// One frame, as it came off the scanner
///
/// Linear 16-bit ADC counts, not display-referred: applying a transfer curve is the caller's.
#[gen_stub_pyclass]
#[pyclass(name = "ScanResult", frozen, get_all, module = "nkscan")]
struct PyScanResult {
    /// (height, width, 3) uint16
    rgb: Py<PyArray3<u16>>,
    /// (height, width) uint16, or None when infrared was not captured
    ir: Option<Py<PyArray2<u16>>>,
    dpi: u32,
    device_model: String,
    frame: usize,
}

/// The two planes a pass produces, as numpy sees them
type Planes = (Py<PyArray3<u16>>, Option<Py<PyArray2<u16>>>);

/// Hand the decoded planes to numpy without copying them
///
/// `into_raw` gives up the buffer that came off the scanner and numpy takes ownership of it. The
/// layout is interleaved and row-major already, which is the shape numpy wants.
fn into_arrays(py: Python<'_>, image: Image) -> PyResult<Planes> {
    let (width, height) = image.rgb.dimensions();
    let rgb = image
        .rgb
        .into_raw()
        .into_pyarray(py)
        .reshape([height as usize, width as usize, 3])?
        .unbind();

    let ir = match image.ir {
        Some(ir) => {
            let (width, height) = ir.dimensions();
            Some(
                ir.into_raw()
                    .into_pyarray(py)
                    .reshape([height as usize, width as usize])?
                    .unbind(),
            )
        }
        None => None,
    };
    Ok((rgb, ir))
}

/// How `prepare`'s arguments name one of the three ways frames get placed
///
/// Pitch is the fallback rather than Sensed: a model that reports no frame table refuses Sensed
/// outright, so inferring it from absent arguments would make the default call the one shape such
/// a model cannot run. Sensed is kept for the transports that do report one, and only while the
/// caller has not overridden the placement with a pitch or an offset.
fn placement_for(
    capabilities: &DeviceCapabilities,
    detected: Option<u32>,
    frames: Option<u32>,
    pitch_mm: Option<f32>,
    offsets_mm: Vec<f32>,
) -> Placement {
    if let Some(frames) = detected {
        return Placement::Detect {
            frames: frames as usize,
        };
    }
    let placed = pitch_mm.is_some() || offsets_mm.iter().any(|&offset| offset != 0.0);
    if capabilities.senses_frames && !placed {
        return Placement::Sensed { frames };
    }
    Placement::Pitch {
        frames,
        pitch_mm,
        offsets_mm,
    }
}

/// An exclusive hold on one scanner
#[gen_stub_pyclass]
#[pyclass(name = "Session", module = "nkscan")]
struct PySession {
    /// `None` once closed, so using a closed session raises rather than panicking
    inner: Mutex<Option<Session>>,
    #[pyo3(get)]
    device_id: String,
    #[pyo3(get)]
    model: String,
}

impl PySession {
    /// Run `body` with the interpreter released, reporting to `progress` as it goes
    ///
    /// The callback re-acquires the interpreter for the moment it runs. Returning `False` from it
    /// stops the pass; anything else, including `None`, keeps going, so a callback that only
    /// prints works. A callback that raises stops the pass and its own exception is what surfaces,
    /// since that is the more interesting failure.
    fn run<T: Send>(
        &self,
        py: Python<'_>,
        progress: Option<Py<PyAny>>,
        body: impl FnOnce(&mut Session, &mut crate::scanners::ProgressFn<'_>) -> Result<T, Error> + Send,
    ) -> PyResult<T> {
        let mut guard = self
            .inner
            .try_lock()
            .map_err(|_| DeviceBusy::new_err("this session is already scanning"))?;
        let session = guard
            .as_mut()
            .ok_or_else(|| ScannerError::new_err("this session is closed"))?;

        let mut last = Instant::now() - PROGRESS_INTERVAL;
        let mut raised: Option<PyErr> = None;

        let outcome = py.detach(|| {
            let mut report = |read: u64, total: u64| {
                // Always report the last chunk, so a bar finishes where it should
                if read < total && last.elapsed() < PROGRESS_INTERVAL {
                    return Flow::Continue;
                }
                last = Instant::now();
                let Some(callback) = progress.as_ref() else {
                    return Flow::Continue;
                };
                Python::attach(|py| match callback.call1(py, (read, total)) {
                    Ok(verdict) => match verdict.extract::<bool>(py) {
                        Ok(false) => Flow::Cancel,
                        _ => Flow::Continue,
                    },
                    Err(err) => {
                        raised = Some(err);
                        Flow::Cancel
                    }
                })
            };
            body(session, &mut report)
        });

        if let Some(err) = raised {
            return Err(err);
        }
        outcome.map_err(to_py)
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PySession {
    /// Claim the scanner `device_id` names, as `list_devices` reported it
    #[new]
    fn new(device_id: &str) -> PyResult<Self> {
        let session = Session::open(device_id).map_err(to_py)?;
        Ok(Self {
            device_id: session.device().id.clone(),
            model: session.device().model.name().to_owned(),
            inner: Mutex::new(Some(session)),
        })
    }

    /// What this unit reports, which is more accurate than what `list_devices` could say
    #[getter]
    fn capabilities(&self, py: Python<'_>) -> PyResult<Py<PyCapabilities>> {
        // Takes a mutable hold now: which adapter is loaded is part of the answer, and reading
        // that is a vendor page inquiry rather than something known at open
        let mut guard = self.inner.lock().expect("no panic holds this");
        let session = guard
            .as_mut()
            .ok_or_else(|| ScannerError::new_err("this session is closed"))?;
        let capabilities = session.capabilities().map_err(to_py)?;
        Py::new(py, PyCapabilities::from(capabilities))
    }

    /// The gain the next pass will use, as `(red, green, blue, ir)`
    #[getter]
    fn gain(&self) -> PyResult<(u32, u32, u32, u32)> {
        let guard = self.inner.lock().expect("no panic holds this");
        let session = guard
            .as_ref()
            .ok_or_else(|| ScannerError::new_err("this session is closed"))?;
        let gain = session.gain();
        Ok((gain.red, gain.green, gain.blue, gain.ir))
    }

    /// Whether there is film in the scanner now
    fn media_loaded(&self, py: Python<'_>) -> PyResult<bool> {
        self.run(py, None, |session, _| session.media_loaded())
    }

    /// How many frames the loaded holder reports, `None` where the model cannot be asked
    ///
    /// A vendor page read rather than a pass, so it answers before the mechanism moves — and on
    /// the models that clear it as the film advances, before the first pass is the only time it
    /// means anything. `0` is an empty transport, which is not the same answer as `None`.
    fn sensed_frames(&self, py: Python<'_>) -> PyResult<Option<u32>> {
        self.run(py, None, |session, _| Ok(session.sensed_frames()))
    }

    /// Wake the mechanism, settle the exposure and place the frames, returning how many
    ///
    /// `detect` asks the scanner to find them with an overview pass, where it can. Otherwise they
    /// are placed at `pitch_mm`, or at whatever the loaded holder reports. `gain` as a 3- or
    /// 4-tuple holds the exposure and skips metering.
    ///
    /// `offsets_mm` shifts each frame along the feed, the last value repeating down the strip, and
    /// takes the place of `offset_mm` when both are given.
    #[pyo3(signature = (
        *, frames = None, detect = false, pitch_mm = None, offset_mm = 0.0, offsets_mm = None,
        gain = None, lock_white_balance = false, wait_for_media_s = 300.0, progress = None,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn prepare(
        &self,
        py: Python<'_>,
        frames: Option<u32>,
        detect: bool,
        pitch_mm: Option<f32>,
        offset_mm: f32,
        offsets_mm: Option<Vec<f32>>,
        gain: Option<Vec<u32>>,
        lock_white_balance: bool,
        wait_for_media_s: f64,
        #[gen_stub(override_type(
            type_repr = "collections.abc.Callable[[int, int], bool | None] | None",
            imports = ("collections.abc")
        ))]
        progress: Option<Py<PyAny>>,
    ) -> PyResult<usize> {
        let offsets_mm = offsets_mm.unwrap_or_else(|| vec![offset_mm]);
        let detected = detect
            .then(|| {
                frames.ok_or_else(|| {
                    PyValueError::new_err("detect needs frames, to say how many to look for")
                })
            })
            .transpose()?;

        let exposure = match gain {
            Some(values) if matches!(values.len(), 3 | 4) => Exposure::Fixed {
                visible: [values[0], values[1], values[2]],
                ir: values.get(3).copied(),
            },
            Some(values) => {
                return Err(PyValueError::new_err(format!(
                    "gain takes three or four values, got {}",
                    values.len()
                )));
            }
            None => Exposure::Auto { lock_white_balance },
        };

        let wait_for_media = Duration::from_secs_f64(wait_for_media_s.max(0.0));
        self.run(py, progress, |session, report| {
            let placement = placement_for(
                &session.capabilities(),
                detected,
                frames,
                pitch_mm,
                offsets_mm,
            );
            session.prepare(
                &Prepare {
                    placement,
                    exposure,
                    wait_for_media,
                },
                report,
            )
        })
    }

    /// Focus, expose and scan one of the placed frames
    #[pyo3(signature = (
        index = 0, *, dpi = 4000, ir = false, focus = "auto", multisample = 1,
        single_line = false, window = None, progress = None,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn scan(
        &self,
        py: Python<'_>,
        index: usize,
        dpi: u16,
        ir: bool,
        focus: &str,
        multisample: u8,
        single_line: bool,
        window: Option<(f32, f32, f32, f32)>,
        #[gen_stub(override_type(
            type_repr = "collections.abc.Callable[[int, int], bool | None] | None",
            imports = ("collections.abc")
        ))]
        progress: Option<Py<PyAny>>,
    ) -> PyResult<PyScanResult> {
        let focus = focus.parse::<FocusMode>().map_err(PyValueError::new_err)?;
        let settings = FrameSettings {
            dpi,
            ir,
            focus,
            multisample,
            single_line,
            window,
        };

        let (image, model) = {
            let model = self.model.clone();
            let image = self.run(py, progress, |session, report| {
                session.scan_frame(index, &settings, report)
            })?;
            (image, model)
        };

        let (rgb, ir) = into_arrays(py, image)?;
        Ok(PyScanResult {
            rgb,
            ir,
            dpi: u32::from(dpi),
            device_model: model,
            frame: index,
        })
    }

    /// The whole strip in one low-resolution pass, as Nikon Scan's thumbnail view
    ///
    /// What a host needs to show the film and let someone pick or place frames on it. `prepare`
    /// with `detect=True` runs the same pass and finds the frames itself; this hands back the
    /// image instead. `dpi` on the result is the pass's own, which is what maps a point on the
    /// thumbnail back to a position on the film.
    #[pyo3(signature = (*, progress = None))]
    fn overview(
        &self,
        py: Python<'_>,
        #[gen_stub(override_type(
            type_repr = "collections.abc.Callable[[int, int], bool | None] | None",
            imports = ("collections.abc")
        ))]
        progress: Option<Py<PyAny>>,
    ) -> PyResult<PyScanResult> {
        let model = self.model.clone();
        let (image, dpi) = self.run(py, progress, |session, report| session.overview(report))?;
        let (rgb, ir) = into_arrays(py, image)?;
        Ok(PyScanResult {
            rgb,
            ir,
            dpi: u32::from(dpi),
            device_model: model,
            frame: 0,
        })
    }

    /// Hold whatever gain the last scan settled on, so the rest of the roll matches it
    fn lock_gain(&self) -> PyResult<()> {
        let mut guard = self.inner.lock().expect("no panic holds this");
        let session = guard
            .as_mut()
            .ok_or_else(|| ScannerError::new_err("this session is closed"))?;
        session.lock_gain();
        Ok(())
    }

    /// Send the film back out
    fn eject(&self, py: Python<'_>) -> PyResult<String> {
        let action = self.run(py, None, |session, _| session.eject())?;
        Ok(match action {
            capability::EjectAction::Unavailable => "none",
            capability::EjectAction::EjectHolder => "holder",
            capability::EjectAction::EjectFilm => "film",
            capability::EjectAction::RewindFilm => "rewind",
            capability::EjectAction::FeedNextSlide => "feed_next",
        }
        .to_owned())
    }

    /// Throw away a pass nobody is going to read, so the session stays usable
    fn abort(&self, py: Python<'_>) -> PyResult<()> {
        self.run(py, None, |session, _| session.abort())
    }

    /// Release the scanner. Idempotent, and using the session afterwards raises.
    fn close(&self) {
        let _ = self.inner.lock().map(|mut guard| guard.take());
    }

    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __exit__(
        &self,
        _type: Option<Py<PyAny>>,
        _value: Option<Py<PyAny>>,
        _traceback: Option<Py<PyAny>>,
    ) -> bool {
        self.close();
        false
    }

    fn __repr__(&self) -> String {
        format!("<nkscan.Session {} {}>", self.device_id, self.model)
    }
}

/// Every scanner attached that this library can drive
///
/// Asks each device who it is and nothing more, so it is safe to call while another process holds
/// one.
#[gen_stub_pyfunction]
#[pyfunction]
fn list_devices(py: Python<'_>) -> PyResult<Vec<PyDevice>> {
    devices::list()
        .into_iter()
        .map(|device: DeviceInfo| {
            Ok(PyDevice {
                id: device.id,
                vendor: devices::VENDOR.to_owned(),
                model: device.model.name().to_owned(),
                capabilities: Py::new(py, PyCapabilities::from(Capabilities::of(device.model)))?,
            })
        })
        .collect()
}

#[pymodule]
#[pyo3(name = "nkscan")]
fn nkscan_module(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(list_devices, module)?)?;
    module.add_class::<PySession>()?;
    module.add_class::<PyDevice>()?;
    module.add_class::<PyCapabilities>()?;
    module.add_class::<PyScanResult>()?;

    let py = module.py();
    module.add("ScannerError", py.get_type::<ScannerError>())?;
    module.add("TransientError", py.get_type::<TransientError>())?;
    module.add("TransportError", py.get_type::<TransportError>())?;
    module.add("DeviceBusy", py.get_type::<DeviceBusy>())?;
    module.add("DeviceNotFound", py.get_type::<DeviceNotFound>())?;
    module.add("MediaError", py.get_type::<MediaError>())?;
    module.add("UnsupportedError", py.get_type::<UnsupportedError>())?;
    module.add("ScanCancelled", py.get_type::<ScanCancelled>())?;
    Ok(())
}

// Reads pyproject.toml for the module name, so `cargo run --bin stub_gen` writes the stub the
// wheel ships
define_stub_info_gatherer!(stub_info);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devices::Model;

    fn placement(model: Model, pitch_mm: Option<f32>, offsets_mm: Vec<f32>) -> Placement {
        placement_for(&model.capabilities(), None, Some(6), pitch_mm, offsets_mm)
    }

    /// The LS-50 refuses Sensed, so the plain call has to reach Pitch
    #[test]
    fn a_model_that_senses_nothing_is_placed_by_pitch() {
        assert_eq!(
            placement(Model::Ls50, None, vec![0.0]),
            Placement::Pitch {
                frames: Some(6),
                pitch_mm: None,
                offsets_mm: vec![0.0],
            }
        );
    }

    #[test]
    fn a_sensing_model_keeps_its_own_table() {
        assert_eq!(
            placement(Model::Ls5000, None, vec![0.0]),
            Placement::Sensed { frames: Some(6) }
        );
    }

    /// A pitch or an offset is a placement the caller chose, and outranks the sensed table
    #[test]
    fn an_explicit_placement_overrides_sensing() {
        assert_eq!(
            placement(Model::Ls5000, Some(38.0), vec![0.0]),
            Placement::Pitch {
                frames: Some(6),
                pitch_mm: Some(38.0),
                offsets_mm: vec![0.0],
            }
        );
        assert!(matches!(
            placement(Model::Ls5000, None, vec![0.0, 0.4]),
            Placement::Pitch { .. }
        ));
    }

    #[test]
    fn detect_wins_wherever_it_is_asked_for() {
        assert_eq!(
            placement_for(
                &Model::Ls9000.capabilities(),
                Some(4),
                Some(6),
                Some(38.0),
                vec![0.2],
            ),
            Placement::Detect { frames: 4 }
        );
    }
}

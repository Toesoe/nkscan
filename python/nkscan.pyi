"""Type stubs for the nkscan extension module.

An abi3 extension carries no introspectable signatures, so this file is what makes the API
discoverable in an editor. Keep it in step with `src/python.rs`.
"""

from types import TracebackType
from typing import Callable, Sequence

import numpy as np
from numpy.typing import NDArray

Progress = Callable[[int, int], bool | None]
"""Called with (bytes read, bytes expected) as a pass runs.

Returning `False` stops the pass and raises `ScanCancelled`. Anything else, `None` included,
keeps going. Reported roughly every 100 ms, which is also how long a cancellation takes to
notice. A pass that has not started reading yet reports (0, 0).
"""

class ScannerError(RuntimeError):
    """Any scanner failure."""

class TransientError(ScannerError):
    """A failure worth retrying: a link glitch, a busy device, a short read."""

class TransportError(TransientError):
    """The link faltered."""

class DeviceBusy(TransientError):
    """Something else holds the scanner, or this session is already scanning."""

class DeviceNotFound(ScannerError):
    """No scanner at that id."""

class MediaError(ScannerError):
    """No film, or nothing recognizable on it. Retrying will not help."""

class UnsupportedError(ScannerError):
    """This scanner does not have that setting."""

class ScanCancelled(ScannerError):
    """The pass was stopped."""

class Capabilities:
    """What a model can do."""

    dpi: list[int]
    """Divisions of the sensor this model offers, coarsest last."""
    depths: list[int]
    """Bits per sample. Always 16 on the wire, whatever is written out."""
    multisample: list[int]
    """Repeat counts, `[1]` where repeats are not driven."""
    ir_channel: bool
    max_area_mm: tuple[float, float]
    """From a `Device` this is the film the model takes; from a `Session` it is what the loaded
    adapter allows, which is the one to trust."""
    auto_exposure: bool
    """Whether the host meters, which is also what makes a white balance lock possible."""
    frame_control: bool
    detects_frames: bool
    """Whether the model can find frames itself, with an overview pass."""
    senses_frames: bool
    """Whether the transport reports where the frames are."""
    single_line: bool
    can_eject: bool

class Device:
    """A scanner the search found, not yet opened."""

    id: str
    """`<model>@<location>`, as `Session` takes it.

    Names the attachment point, not the unit: these scanners report no serial number, so an id
    holds only until something is replugged. Re-enumerate rather than storing one.
    """
    vendor: str
    model: str
    capabilities: Capabilities

class ScanResult:
    """One frame, as it came off the scanner.

    Linear 16-bit ADC counts, not display-referred: applying a transfer curve is the caller's.
    """

    rgb: NDArray[np.uint16]
    """(height, width, 3), owning the buffer the scanner produced."""
    ir: NDArray[np.uint16] | None
    """(height, width), or None when infrared was not captured."""
    dpi: int
    device_model: str
    frame: int

class Session:
    """An exclusive hold on one scanner: opened once, used for as many frames as the film has."""

    device_id: str
    model: str
    capabilities: Capabilities
    gain: tuple[int, int, int, int]
    """The per-channel analog gain the next pass will use, as (red, green, blue, ir)."""

    def __init__(self, device_id: str) -> None: ...
    def media_loaded(self) -> bool:
        """Whether there is film in the scanner now."""

    def prepare(
        self,
        *,
        frames: int | None = None,
        detect: bool = False,
        pitch_mm: float | None = None,
        offset_mm: float = 0.0,
        gain: Sequence[int] | None = None,
        lock_white_balance: bool = False,
        wait_for_media_s: float = 300.0,
        progress: Progress | None = None,
    ) -> int:
        """Wake the mechanism, settle the exposure and place the frames, returning how many.

        `detect` asks the scanner to find them with an overview pass, where it can, and needs
        `frames` to say how many to look for. Otherwise they are placed at `pitch_mm`, or at
        whatever the loaded holder reports. `gain` as three or four values holds the exposure
        and skips metering entirely.
        """

    def scan(
        self,
        index: int = 0,
        *,
        dpi: int = 4000,
        ir: bool = False,
        focus: str = "auto",
        multisample: int = 1,
        single_line: bool = False,
        window: tuple[float, float, float, float] | None = None,
        progress: Progress | None = None,
    ) -> ScanResult:
        """Focus, expose and scan one of the placed frames.

        `focus` is `"auto"` or a setpoint in the scanner's own units. `window` is not
        implemented and raises `UnsupportedError` rather than being ignored.
        """

    def lock_gain(self) -> None:
        """Hold whatever gain the last scan settled on, so the rest of the roll matches it."""

    def eject(self) -> bool:
        """Send the film back out."""

    def abort(self) -> None:
        """Throw away a pass nobody is going to read, so the session stays usable."""

    def close(self) -> None:
        """Release the scanner. Idempotent, and using the session afterwards raises."""

    def __enter__(self) -> "Session": ...
    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc: BaseException | None,
        traceback: TracebackType | None,
    ) -> bool: ...

def list_devices() -> list[Device]:
    """Every scanner attached that this library can drive.

    Asks each device who it is and nothing more, so it is safe to call while another process
    holds one.
    """

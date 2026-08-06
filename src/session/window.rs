//! GET WINDOW, SET WINDOW and SCAN. Sections 2-9, 2-10 and 2-7

use super::{MOVE_TIMEOUT, PROBE_TIMEOUT, Session, malformed};
use crate::{
    error::Error,
    protocol::{
        cdbs::{GetWindow, Scan, SetWindow},
        data::CooperativeAction,
        image::Layout,
        window::{self, GetWindowHeader, SetWindowHeader, Window},
    },
    transport::Data,
};
use tracing::*;

/// A unit that asks forever is not going to start, so cap the re-issues
const MAX_COOPERATION: usize = 4;

impl Session {
    /// One GET WINDOW, exactly as asked: header plus descriptors, unparsed
    fn get_window_raw(&mut self, cmd: GetWindow) -> Result<Vec<u8>, Error> {
        let mut buf = vec![0u8; cmd.allocation_length()];
        let completion = self.run(&cmd.cdb(), Data::In(&mut buf), PROBE_TIMEOUT)?;
        buf.truncate(completion.transferred);
        Ok(buf)
    }

    /// Read back every window descriptor the unit currently holds
    ///
    /// Two passes: a transfer longer than what is there gets refused, so the
    /// header has to say how much there is first
    pub fn windows(&mut self) -> Result<Vec<Window>, Error> {
        let probe = self.get_window_raw(GetWindow::all(window::HEADER as u32))?;
        let (probe, _) =
            GetWindowHeader::from_bytes(&probe).map_err(|e| malformed(e.to_string()))?;

        let data = self.get_window_raw(GetWindow::all(2 + u32::from(probe.data_length)))?;
        let (header, descriptors) =
            GetWindowHeader::from_bytes(&data).map_err(|e| malformed(e.to_string()))?;
        let stride = usize::from(header.descriptor_length);
        debug!(stride, bytes = descriptors.len(), "window descriptors");

        if stride < window::LENGTH {
            return Err(malformed(format!(
                "descriptor stride of {stride} is shorter than the {} bytes 2-10-3 defines",
                window::LENGTH
            )));
        }

        descriptors
            .chunks_exact(stride)
            .map(|d| Window::try_from(d).map_err(|e| malformed(e.to_string())))
            .collect()
    }

    /// Define one window
    pub fn set_window(&mut self, window: &Window) -> Result<(), Error> {
        window.validate(&self.caps)?;

        let header = SetWindowHeader {
            descriptor_length: window::LENGTH as u16,
        };
        let mut payload = Vec::with_capacity(window::HEADER + window::LENGTH);
        payload.extend_from_slice(&header.to_bytes());
        payload.extend_from_slice(&window.to_bytes());

        let cmd = SetWindow::new(payload.len() as u32);
        debug!(id = window.id, "setting window");
        self.run(&cmd.cdb(), Data::Out(&payload), MOVE_TIMEOUT)?;
        Ok(())
    }

    /// Scan the windows named, and hand back what it will produce
    ///
    /// 2-7: the unit answers, then scans, so this returns once it has started
    /// and [`test_unit_ready`](Session::test_unit_ready) says when the data is
    /// there.
    ///
    /// A cooperative request is not a blocker. The captures read the
    /// `DataType::Cooperation` record and
    /// send SCAN again with nothing in between: it says what the host will owe
    /// the *data*, not what has to happen before the scan runs. Whatever it
    /// asks for comes back on [`Started::cooperation`] for the caller to honor
    /// once the image is read.
    pub fn scan(&mut self, windows: &[Window]) -> Result<Started, Error> {
        // Checks every rule spanning the set on the way
        let layout = Layout::new(&self.caps, windows, self.divisor)?;

        let ids: Vec<u8> = windows.iter().map(|w| w.id).collect();
        let cmd = Scan::new(ids.len() as u8);
        let mut cooperation = None;

        for attempt in 0..=MAX_COOPERATION {
            let (_, coop) = self.run_cooperative(&cmd.cdb(), Data::Out(&ids), MOVE_TIMEOUT)?;
            let Some(coop) = coop else {
                debug!(?ids, ?cooperation, "scanning");
                return Ok(Started {
                    layout,
                    cooperation,
                });
            };

            // Dispatch on the record rather than the sense: the two specs give
            // the same job different 4th sense bytes
            let record = self.cooperation()?;
            debug!(?coop, ?record, attempt, "the unit wants something doing");
            cooperation = Some(record);
        }

        Err(Error::Unsupported {
            op: "host cooperation",
            reason: format!("the unit kept asking after {MAX_COOPERATION} re-issues"),
        })
    }
}

/// A scan that has started
#[derive(Debug, Clone)]
pub struct Started {
    /// What the stream will look like
    pub layout: Layout,
    /// What the unit asked the host to do with the data, if anything. Reading
    /// the record is what lets the scan proceed; honoring it is the caller's
    pub cooperation: Option<CooperativeAction>,
}

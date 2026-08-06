//! GET WINDOW, SET WINDOW and SCAN. Sections 2-9, 2-10 and 2-7

use super::{MOVE_TIMEOUT, PROBE_TIMEOUT, Session, malformed};
use crate::{
    error::Error,
    protocol::{
        cdbs::{GetWindow, Scan, SetWindow},
        image::Layout,
        window::{self, GetWindowHeader, SetWindowHeader, Window},
    },
    transport::Data,
};
use tracing::*;

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

    /// Scan the windows named, and hand back the layout of what it will produce
    ///
    /// 2-7: the unit answers, then scans, so this returns once it has started
    /// and [`test_unit_ready`](Session::test_unit_ready) says when the data is
    /// there. A cooperative request means it will not start until the initiator
    /// has done a job for it, named by the `87h` record
    pub fn scan(&mut self, windows: &[Window]) -> Result<Layout, Error> {
        let layout = Layout::new(&self.caps, windows, self.divisor)?;
        check_ordering(windows)?;

        let ids: Vec<u8> = windows.iter().map(|w| w.id).collect();
        let cmd = Scan::new(ids.len() as u8);
        let (_, coop) = self.run_cooperative(&cmd.cdb(), Data::Out(&ids), MOVE_TIMEOUT)?;

        let Some(coop) = coop else {
            debug!(?ids, "scanning");
            return Ok(layout);
        };

        // Read the parameter anyway: it names the job precisely, where the 4th
        // sense byte the two specs disagree about does not. When the first job
        // is implemented this becomes a loop: do the work, issue SCAN again
        let record = self.cooperation()?;
        Err(Error::Unsupported {
            op: "host cooperation",
            reason: format!("{coop:?} is not implemented yet, and it wants {record:?}"),
        })
    }
}

/// 2-10 byte 40: across a window set the read positions must be all zero, or all
/// nonzero with no repeats. SCAN answers anything else with `05h-2Ch-02h`
fn check_ordering(windows: &[Window]) -> Result<(), Error> {
    let bad = |reason: String| Error::Unsupported {
        op: "color ordering",
        reason,
    };
    let orders: Vec<u8> = windows.iter().map(|w| w.color_ordering).collect();

    if orders.iter().all(|&o| o == 0) {
        return Ok(());
    }
    if let Some(w) = windows.iter().find(|w| w.color_ordering == 0) {
        return Err(bad(format!(
            "window {} leaves the order to the unit while the rest of the set pins it",
            w.id
        )));
    }
    for (n, &order) in orders.iter().enumerate() {
        if orders[..n].contains(&order) {
            return Err(bad(format!("read position {order} is claimed twice")));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(orders: &[u8]) -> Vec<Window> {
        orders
            .iter()
            .enumerate()
            .map(|(n, &order)| {
                let mut w = Window::try_from(&[0u8; window::LENGTH][..]).unwrap();
                w.id = n as u8 + 1;
                w.color_ordering = order;
                w
            })
            .collect()
    }

    #[test]
    fn a_window_set_orders_every_color_or_none() {
        assert!(check_ordering(&set(&[0, 0, 0])).is_ok());
        assert!(check_ordering(&set(&[1, 2, 3])).is_ok());
        assert!(check_ordering(&set(&[1, 0, 3])).is_err());
        assert!(check_ordering(&set(&[1, 2, 2])).is_err());
    }
}

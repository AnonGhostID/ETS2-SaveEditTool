//! One-shot background loader relocated unchanged from `gui/mod.rs:141-183`.

/// A one-shot background load: work runs on a plain OS thread and the
/// result is picked up by polling `poll()` from the UI thread. While a
/// load is running, `is_loading()` is true and the UI shows a spinner.
pub struct BackgroundLoad<T> {
    receiver: Option<std::sync::mpsc::Receiver<T>>,
}

impl<T> Default for BackgroundLoad<T> {
    fn default() -> Self {
        Self { receiver: None }
    }
}

impl<T: Send + 'static> BackgroundLoad<T> {
    pub fn is_loading(&self) -> bool {
        self.receiver.is_some()
    }

    /// Start `work` on a background thread. Any previous load is dropped
    /// (its result is discarded when it arrives).
    pub fn start<F: FnOnce() -> T + Send + 'static>(&mut self, work: F) {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(work());
        });
        self.receiver = Some(rx);
    }

    /// Pick up a finished result, if one arrived. Returns `None` while
    /// the load is still running (or when no load is running).
    pub fn poll(&mut self) -> Option<T> {
        let receiver = self.receiver.as_ref()?;
        match receiver.try_recv() {
            Ok(value) => {
                self.receiver = None;
                Some(value)
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.receiver = None;
                None
            }
        }
    }
}

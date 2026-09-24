use coroflow::{MutableSharedFlow, SharedFlow};

/// How many messages wait for a screen that is not showing them yet.
const MESSAGE_BUFFER: usize = 8;

/// One-shot messages, such as a toast, for the screen that owns this bus.
///
/// It lives in the screen's view model store, so the screen and every part
/// of it that resolves a view model there get the same bus without knowing
/// about each other: the arch starter's `ScreenBus`.
pub struct ScreenMessages {
    messages: MutableSharedFlow<String>,
}

impl Default for ScreenMessages {
    fn default() -> Self {
        Self {
            messages: MutableSharedFlow::new(0, MESSAGE_BUFFER),
        }
    }
}

impl ScreenMessages {
    /// The messages, as they are sent.
    pub fn events(&self) -> SharedFlow<String> {
        self.messages.as_shared_flow()
    }

    /// Sends `message` to whoever shows this screen's messages.
    pub fn send(&self, message: String) {
        self.messages.try_emit(message);
    }
}

#[derive(Debug, Clone, Default)]
pub struct State {
    pub last_card_id: Option<String>,
    /// Card bytes queued by the GUI for emulated POLL responses.
    /// Consumed once by the emulator, then cleared.
    pub pending_card: Option<[u8; 8]>,
}
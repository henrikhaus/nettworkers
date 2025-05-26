#[derive(Debug, Clone, PartialEq, Default)]
pub struct SettingsState {
    pub delay: u64,
}

impl SettingsState {
    pub fn new() -> Self {
        SettingsState { delay: 0 }
    }

    pub fn set_delay(&mut self, delay: u64) {
        self.delay = delay;
    }

    pub fn get_delay(&self) -> u64 {
        self.delay
    }
}

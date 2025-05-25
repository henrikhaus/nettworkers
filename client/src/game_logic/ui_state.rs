use crate::SERVER_ADDR;

/// UI state machine: manages which screen is active and navigation history

/// All possible UI screens
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    MainMenu,
    InGame,
    PauseMenu,
    Settings,
    Disconnecting,
    Connecting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MainMenuInput {
    #[default]
    None,
    ServerIp,
    PlayerName,
}

/// UIState holds a stack of screens to allow push/pop navigation
#[derive(Debug, Default)]
pub struct UiState {
    stack: Vec<Screen>,
    pub server_ip: String,
    pub ip_focused: bool,
    pub player_name: String,
    pub name_focused: bool,
    pub focused_input: MainMenuInput,
}

impl UiState {
    /// Create a new UiState starting at MainMenu
    pub fn new() -> UiState {
        UiState {
            stack: vec![Screen::MainMenu],
            server_ip: String::from(SERVER_ADDR),
            ip_focused: false,
            player_name: String::new(),
            name_focused: false,
            focused_input: MainMenuInput::None,
        }
    }

    /// Get a reference to the current screen
    pub fn current_screen(&self) -> &Screen {
        self.stack
            .last()
            .expect("UiState stack should never be empty")
    }

    /// Push a new screen onto the stack
    pub fn push(&mut self, screen: Screen) {
        self.stack.push(screen);
    }

    /// Pop the current screen, returning to the previous one if available
    pub fn pop(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    /// Replace the current screen with a new one
    pub fn replace(&mut self, screen: Screen) {
        if let Some(last) = self.stack.last_mut() {
            *last = screen;
        }
    }

    /// Clear history and set a single screen
    pub fn reset(&mut self, screen: Screen) {
        self.stack.clear();
        self.stack.push(screen);
    }

    /// Check if a screen is anywhere in the stack
    pub fn contains(&self, screen: &Screen) -> bool {
        self.stack.contains(screen)
    }
}

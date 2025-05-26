/// UI state machine: manages which screen is active and navigation history

/// All possible UI screens
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    MainMenu,
    InGame,
    PauseMenu,
    Settings,
    // add more screens as needed
}

/// UIState holds a stack of screens to allow push/pop navigation
#[derive(Debug, Default)]
pub struct UiState {
    stack: Vec<Screen>,
}

impl UiState {
    /// Create a new UiState starting at MainMenu
    pub fn new() -> Self {
        UiState {
            stack: vec![Screen::MainMenu],
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

    /// Clear history and set a single screen
    pub fn reset(&mut self, screen: Screen) {
        self.stack.clear();
        self.stack.push(screen);
    }
}

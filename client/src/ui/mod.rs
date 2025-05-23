pub mod context;
pub mod layout;
pub mod screens;
pub mod style;
pub mod widget;

// Re-export the core UI pieces for ergonomic imports:
pub use context::{DrawCmd, UiContext};
pub use layout::{HBox, VBox /*, Grid, Layout if you have them */};
pub use style::Theme;
pub use widget::{Button, Label, UiResponse, Widget};
// screens:
pub use screens::{hud, main_menu, pause_menu};

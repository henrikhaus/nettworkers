use macroquad::prelude::Color;

/// Visual theme colors for UI widgets.
#[derive(Clone, Debug)]
pub struct Theme {
    /// Primary text color
    pub text_color:     Color,
    /// Background color for buttons
    pub button_bg:      Color,
    /// Background color for hovered buttons
    pub button_hover_bg: Color,
    /// Background color for panels/dialogs
    pub panel_bg:       Color,
    /// Fallback window/background color
    pub window_bg:      Color,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            text_color:      Color::from_rgba(255, 255, 255, 255), // white
            button_bg:       Color::from_rgba(40,  40,  40,  200), // dark grey
            button_hover_bg: Color::from_rgba(60,  60,  60,  200), // lighter grey
            panel_bg:        Color::from_rgba(20,  20,  20,  220), // almost black
            window_bg:       Color::from_rgba(0,    0,   0,  255), // black
        }
    }
}

impl Theme {
    /// Create a theme from custom color values.
    pub fn new(
        text_color: Color,
        button_bg: Color,
        button_hover_bg: Color,
        panel_bg: Color,
        window_bg: Color,
    ) -> Self {
        Theme { text_color, button_bg, button_hover_bg, panel_bg, window_bg }
    }
}
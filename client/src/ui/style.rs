use macroquad::prelude::Color;

/// Visual theme colors for UI widgets.
pub struct Theme {
    /// Primary text color
    pub text_color: Color,
    /// Background color for buttons
    pub button_bg: Color,
    /// Background color for hovered buttons
    pub button_hover_bg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            text_color: Color::from_rgba(255, 255, 255, 255), // white
            button_bg: Color::from_rgba(40, 40, 40, 200),     // dark grey
            button_hover_bg: Color::from_rgba(60, 60, 60, 200), // lighter grey
        }
    }
}

impl Theme {}

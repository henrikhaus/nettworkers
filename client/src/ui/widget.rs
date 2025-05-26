use crate::ui::context::DrawCmd;
use crate::ui::context::UiContext;
use macroquad::math::{Rect, Vec2};

/// Response from widget interaction
#[derive(Debug, PartialEq, Eq)]
pub enum UiResponse {
    None,
    Clicked,
}

/// Trait every UI widget implements.
pub trait Widget {
    /// Handle input, update UI state, and record draw commands.
    fn ui(&mut self, ctx: &mut UiContext, area: Rect) -> UiResponse;
}

/// Simple text label.
#[derive(Debug, Clone)]
pub struct Label {
    pub text: String,
}

impl Label {
    pub fn new<T: Into<String>>(text: T) -> Self {
        Label { text: text.into() }
    }
}

impl Widget for Label {
    fn ui(&mut self, ctx: &mut UiContext, area: Rect) -> UiResponse {
        ctx.push_cmd(DrawCmd::Text {
            text: self.text.clone(),
            pos: Vec2::new(area.x, area.y + ctx.font_size),
            font_size: ctx.font_size,
            color: ctx.theme.text_color,
        });
        UiResponse::None
    }
}

/// A clickable button with a label and optional callback.
pub struct Button {
    pub label: String,
    pub on_click: Option<Box<dyn FnMut()>>,
}

impl Button {
    pub fn new<T: Into<String>>(label: T) -> Self {
        Button {
            label: label.into(),
            on_click: None,
        }
    }

    pub fn with_callback<F: 'static + FnMut()>(mut self, cb: F) -> Self {
        self.on_click = Some(Box::new(cb));
        self
    }
}

// Custom Debug impl omits callback
impl std::fmt::Debug for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Button")
            .field("label", &self.label)
            .finish()
    }
}

impl Widget for Button {
    fn ui(&mut self, ctx: &mut UiContext, area: Rect) -> UiResponse {
        // change button color when hovered
        let bg = if area.contains(ctx.mouse_pos) && ctx.mouse_down {
            ctx.theme.button_hover_bg
        } else {
            ctx.theme.button_bg
        };
        ctx.push_cmd(DrawCmd::Rect {
            rect: area,
            color: bg,
        });

        // center text within button
        let text_w = self.label.len() as f32 * ctx.font_size * 0.5;
        let text_h = ctx.font_size;
        let pos = Vec2::new(
            area.x + (area.w - text_w) / 2.0,
            area.y + (area.h + text_h) / 2.0,
        );
        ctx.push_cmd(DrawCmd::Text {
            text: self.label.clone(),
            pos,
            font_size: ctx.font_size,
            color: ctx.theme.text_color,
        });

        // click detection
        if area.contains(ctx.mouse_pos) && ctx.mouse_down {
            if let Some(cb) = &mut self.on_click {
                cb();
            }
            return UiResponse::Clicked;
        }
        UiResponse::None
    }
}

/// A toggle switch that can be either on or off.
pub struct Toggle {
    pub is_on: bool,
    pub label: String,
    pub on_change: Option<Box<dyn FnMut(bool)>>,
}

impl Toggle {
    pub fn new<T: Into<String>>(label: T) -> Self {
        Toggle {
            label: label.into(),
            is_on: false,
            on_change: None,
        }
    }

    pub fn with_state(mut self, initial_state: bool) -> Self {
        self.is_on = initial_state;
        self
    }

    pub fn with_callback<F: 'static + FnMut(bool)>(mut self, cb: F) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }
}

// Custom Debug impl omits callback
impl std::fmt::Debug for Toggle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Toggle")
            .field("is_on", &self.is_on)
            .field("label", &self.label)
            .finish()
    }
}

impl Widget for Toggle {
    fn ui(&mut self, ctx: &mut UiContext, area: Rect) -> UiResponse {
        let toggle_width = 40.0;
        let toggle_height = 20.0;
        let padding = 8.0;

        // Draw label
        ctx.push_cmd(DrawCmd::Text {
            text: self.label.clone(),
            pos: Vec2::new(area.x, area.y + ctx.font_size),
            font_size: ctx.font_size,
            color: ctx.theme.text_color,
        });

        // Calculate toggle area (to the right of the label)
        let toggle_area = Rect::new(
            area.x + area.w - toggle_width,
            area.y + (area.h - toggle_height) / 2.0,
            toggle_width,
            toggle_height,
        );

        // Draw toggle background
        let bg_color = if self.is_on {
            ctx.theme.button_hover_bg
        } else {
            ctx.theme.button_bg
        };
        ctx.push_cmd(DrawCmd::Rect {
            rect: toggle_area,
            color: bg_color,
        });

        // Draw toggle knob
        let knob_size = toggle_height - padding;
        let knob_x = if self.is_on {
            toggle_area.x + toggle_width - knob_size - padding / 2.0
        } else {
            toggle_area.x + padding / 2.0
        };

        ctx.push_cmd(DrawCmd::Rect {
            rect: Rect::new(knob_x, toggle_area.y + padding / 2.0, knob_size, knob_size),
            color: ctx.theme.text_color,
        });

        // Handle click
        if toggle_area.contains(ctx.mouse_pos) && ctx.mouse_down {
            self.is_on = !self.is_on;
            if let Some(cb) = &mut self.on_change {
                cb(self.is_on);
            }
            return UiResponse::Clicked;
        }
        UiResponse::None
    }
}

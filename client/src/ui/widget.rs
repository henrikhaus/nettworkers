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

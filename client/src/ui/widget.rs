use crate::ui::context::DrawCmd;
use crate::ui::context::UiContext;
use macroquad::input::{KeyCode, get_char_pressed, is_key_down, is_key_pressed};
use macroquad::math::{Rect, Vec2, vec2};
use macroquad::text::measure_text;
use macroquad::time::get_time;

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

/// A single-line text input.
pub struct TextInput<'a> {
    /// The backing string to edit
    pub text: &'a mut String,
    /// Whether this field has focus
    pub focused: &'a mut bool,
    /// Placeholder text when `text` is empty
    pub placeholder: String,
}

impl<'a> TextInput<'a> {
    /// Create a new text input widget.
    pub fn new<T: Into<String>>(
        text: &'a mut String,
        focused: &'a mut bool,
        placeholder: T,
    ) -> Self {
        TextInput {
            text,
            focused,
            placeholder: placeholder.into(),
        }
    }
}

// in ui/widget.rs
impl<'a> Widget for TextInput<'a> {
    fn ui(&mut self, ctx: &mut UiContext, area: Rect) -> UiResponse {
        // 1) Draw border (thicker/brighter when focused)
        let border_color = if *self.focused {
            ctx.theme.focus_border
        } else {
            ctx.theme.panel_border
        };
        // outer border
        ctx.push_cmd(DrawCmd::Rect {
            rect: area,
            color: border_color,
        });
        // inner background (inset by 1px)
        let bg = Rect::new(area.x + 1.0, area.y + 1.0, area.w - 2.0, area.h - 2.0);
        ctx.push_cmd(DrawCmd::Rect {
            rect: area,
            color: ctx.theme.panel_bg,
        });

        // 2) Handle clicking on/off
        if ctx.mouse_down {
            *self.focused = area.contains(ctx.mouse_pos);
        }

        // 3) Draw text or placeholder
        let display = if self.text.is_empty() && !*self.focused {
            &self.placeholder
        } else {
            &self.text
        };
        let mut color = ctx.theme.text_color;
        if self.text.is_empty() {
            color.a *= 0.5;
        }
        ctx.push_cmd(DrawCmd::Text {
            text: display.clone(),
            pos: vec2(area.x + 4.0, area.y + ctx.font_size + 2.0),
            font_size: ctx.font_size,
            color,
        });

        // 4) If focused, grab every keystroke & backspace
        if *self.focused {
            while let Some(c) = get_char_pressed() {
                if !c.is_control() {
                    self.text.push(c);
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                self.text.pop();
            }
        }

        if *self.focused {
            let dims = measure_text(
                &self.text,
                None,                 // use default font
                ctx.font_size as u16, // font size in px
                1.0,                  // scale
            );
            // caret x = left padding + text width
            let caret_x = area.x + 4.0 + dims.width;
            // caret y = baseline same as text
            let caret_y = area.y + ctx.font_size + 2.0;

            // blink period: toggle every 0.5s
            if (get_time() % 1.0) < 0.5 {
                ctx.push_cmd(DrawCmd::Text {
                    text: "|".to_string(),
                    pos: vec2(caret_x, caret_y),
                    font_size: ctx.font_size,
                    color: ctx.theme.text_color,
                });
            }
        }

        UiResponse::None
    }
}

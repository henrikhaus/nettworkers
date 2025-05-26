use crate::ui::style::Theme;
use macroquad::math::Vec2;
use macroquad::prelude::*;

/// A drawing command recorded by the UI.  When executed, issues the actual draw calls.
pub enum DrawCmd {
    Rect {
        rect: Rect,
        color: Color,
    },
    Text {
        text: String,
        pos: Vec2,
        font_size: f32,
        color: Color,
    },
    // extend with Image, Lines, etc. as needed
}

impl DrawCmd {
    /// Execute this draw command immediately using macroquad
    pub fn execute(&self) {
        match self {
            DrawCmd::Rect { rect, color } => {
                draw_rectangle(rect.x, rect.y, rect.w, rect.h, *color);
            }
            DrawCmd::Text {
                text,
                pos,
                font_size,
                color,
            } => {
                draw_text(text, pos.x, pos.y, *font_size, *color);
            }
        }
    }
}

/// The UI context drives all immediate‐mode drawing, input capture, and ID generation.
pub struct UiContext {
    pub mouse_pos: Vec2,
    pub mouse_down: bool,
    pub font_size: f32,
    pub theme: Theme,
    draw_commands: Vec<DrawCmd>,
}

impl UiContext {
    /// Create a new UI context with default theme, font size, and empty draw list.
    pub fn new() -> Self {
        Self {
            mouse_pos: Vec2::ZERO,
            mouse_down: false,
            font_size: 16.0,
            theme: Theme::default(),
            draw_commands: Vec::new(),
        }
    }

    /// Begin a new frame: sample input and clear pending draw commands
    pub fn begin_frame(&mut self) {
        self.capture_input();
        self.draw_commands.clear();
    }

    /// Record a drawing command (rectangle, text, etc.)
    pub fn push_cmd(&mut self, cmd: DrawCmd) {
        self.draw_commands.push(cmd);
    }

    /// Flush all recorded draw commands to the screen
    pub fn end_frame(&mut self) {
        for cmd in &self.draw_commands {
            cmd.execute();
        }
    }

    /// Internal: sample mouse position & button state
    fn capture_input(&mut self) {
        let (x, y) = mouse_position();
        self.mouse_pos = Vec2::new(x, y);
        self.mouse_down = is_mouse_button_pressed(MouseButton::Left);
    }
}

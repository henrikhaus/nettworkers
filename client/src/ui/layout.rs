use crate::ui::context::UiContext;
use macroquad::math::{Rect, Vec2};

/// Horizontal stacking layout: arranges items left-to-right
pub struct HBox {
    pub padding: f32,
    pub spacing: f32,
    state: Option<HBoxState>,
}

struct HBoxState {
    area: Rect,
    cursor: Vec2,
}

impl HBox {
    /// Create a new HBox with given padding and spacing
    pub fn new(padding: f32, spacing: f32) -> Self {
        Self {
            padding,
            spacing,
            state: None,
        }
    }

    /// Begin laying out items within `area`
    pub fn begin(&mut self, _ui: &mut UiContext, area: Rect) {
        let cursor = Vec2::new(area.x + self.padding, area.y + self.padding);
        self.state = Some(HBoxState { area, cursor });
    }

    /// Reserve a slot for the next item with desired size, returning its Rect.
    pub fn item(&mut self, _ui: &mut UiContext, desired: Vec2) -> Rect {
        let st = self
            .state
            .as_mut()
            .expect("HBox::begin must be called before item");
        // clamp item's height to available height
        let height = (st.area.h - 2.0 * self.padding).max(0.0);
        let size = Vec2::new(desired.x, desired.y.min(height));
        let rect = Rect::new(st.cursor.x, st.cursor.y, size.x, size.y);
        // advance cursor
        st.cursor.x += size.x + self.spacing;
        rect
    }

    /// Finish the layout, clearing internal state
    pub fn end(&mut self, _ui: &mut UiContext) {
        self.state = None;
    }
}

/// Vertical stacking layout: arranges items top-to-bottom
pub struct VBox {
    pub padding: f32,
    pub spacing: f32,
    state: Option<VBoxState>,
}

struct VBoxState {
    area: Rect,
    cursor: Vec2,
}

impl VBox {
    /// Create a new VBox with given padding and spacing
    pub fn new(padding: f32, spacing: f32) -> Self {
        Self {
            padding,
            spacing,
            state: None,
        }
    }

    /// Begin laying out items within `area`
    pub fn begin(&mut self, _ui: &mut UiContext, area: Rect) {
        let cursor = Vec2::new(area.x + self.padding, area.y + self.padding);
        self.state = Some(VBoxState { area, cursor });
    }

    /// Reserve a slot for the next item with desired size, returning its Rect.
    pub fn item(&mut self, _ui: &mut UiContext, desired: Vec2) -> Rect {
        let st = self
            .state
            .as_mut()
            .expect("VBox::begin must be called before item");
        // clamp item's width to available width
        let width = (st.area.w - 2.0 * self.padding).max(0.0);
        let size = Vec2::new(desired.x.min(width), desired.y);
        let rect = Rect::new(st.cursor.x, st.cursor.y, size.x, size.y);
        // advance cursor
        st.cursor.y += size.y + self.spacing;
        rect
    }

    /// Finish the layout, clearing internal state
    pub fn end(&mut self, _ui: &mut UiContext) {
        self.state = None;
    }
}

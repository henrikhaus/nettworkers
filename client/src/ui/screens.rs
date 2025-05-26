use crate::Scene;
use crate::game_logic::{Screen, UiState};
use crate::state::{GameState, PlayerState};
use crate::ui::{Button, DrawCmd, Label, UiContext, UiResponse, VBox, Widget};
use macroquad::math::{Rect, vec2};
use macroquad::time::{get_fps, get_time};
use macroquad::window::{screen_height, screen_width, set_fullscreen};

use super::widget::Toggle;

// throttle FPS meter updates to every 0.2 seconds
static mut LAST_FPS_UPDATE: f64 = 0.0;
static mut DISPLAY_FPS: f32 = 0.0;
const FPS_UPDATE_INTERVAL: f64 = 0.2;

/// Main menu screen: shows title and navigation buttons.
pub fn main_menu(ctx: &mut UiContext, state: &mut UiState) {
    let area = Rect::new(0.0, 0.0, screen_width(), screen_height());
    let mut menu = VBox::new(20.0, 10.0);
    menu.begin(ctx, area);

    let title_area = menu.item(ctx, vec2(300.0, 50.0));
    Label::new("My Rust Game").ui(ctx, title_area);

    let start_area = menu.item(ctx, vec2(200.0, 50.0));
    if Button::new("Start Game").ui(ctx, start_area) == UiResponse::Clicked {
        state.push(Screen::InGame);
    }

    let settings_area = menu.item(ctx, vec2(200.0, 50.0));
    if Button::new("Settings").ui(ctx, settings_area) == UiResponse::Clicked {
        state.push(Screen::Settings);
    }

    let quit_area = menu.item(ctx, vec2(200.0, 50.0));
    if Button::new("Quit").ui(ctx, quit_area) == UiResponse::Clicked {
        std::process::exit(0);
    }

    menu.end(ctx);
}

/// Settings menu: adjust game options
pub fn settings_menu(
    ctx: &mut UiContext,
    state: &mut UiState,
    delay: bool,
    reconciliation: bool,
    prediction: bool,
    mut on_delay_change: impl FnMut() -> (),
    mut on_reconciliation_change: impl FnMut() -> (),
    mut on_prediction_change: impl FnMut() -> (),
) {
    let area = Rect::new(0.0, 0.0, screen_width(), screen_height());
    let mut menu = VBox::new(20.0, 10.0);
    menu.begin(ctx, area);

    let title_area = menu.item(ctx, vec2(300.0, 50.0));
    Label::new("Settings").ui(ctx, title_area);

    let fullscreen_area = menu.item(ctx, vec2(200.0, 50.0));
    if Button::new("Fullscreen mode").ui(ctx, fullscreen_area) == UiResponse::Clicked {
        set_fullscreen(true);
    }

    let back_area = menu.item(ctx, vec2(200.0, 50.0));
    if Button::new("Back").ui(ctx, back_area) == UiResponse::Clicked {
        state.pop();
    }

    let reconciliation_toggle_area = menu.item(ctx, vec2(200.0, 50.0));
    if Toggle::new("Reconciliation")
        .with_state(reconciliation)
        .ui(ctx, reconciliation_toggle_area)
        == UiResponse::Clicked
    {
        on_reconciliation_change();
    }

    let prediction_toggle_area = menu.item(ctx, vec2(200.0, 50.0));
    if Toggle::new("Prediction")
        .with_state(prediction)
        .ui(ctx, prediction_toggle_area)
        == UiResponse::Clicked
    {
        on_prediction_change();
    }

    let delay_toggle_area = menu.item(ctx, vec2(200.0, 50.0));
    if Toggle::new("1000ms delay")
        .with_state(delay)
        .ui(ctx, delay_toggle_area)
        == UiResponse::Clicked
    {
        on_delay_change();
    }

    menu.end(ctx);
}

/// In-game HUD: shows FPS and player count, updated at a fixed interval
pub fn hud(ctx: &mut UiContext, state: &mut UiState, game_state: &GameState, _scene: &Scene) {
    // throttle FPS updates
    let now = get_time();
    unsafe {
        if now - LAST_FPS_UPDATE > FPS_UPDATE_INTERVAL {
            DISPLAY_FPS = get_fps() as f32;
            LAST_FPS_UPDATE = now;
        }
    }
    let fps_display = unsafe { DISPLAY_FPS };
    let fps_text = format!("FPS: {:.0}", fps_display);
    ctx.push_cmd(DrawCmd::Text {
        text: fps_text,
        pos: vec2(10.0, 20.0),
        font_size: ctx.font_size,
        color: ctx.theme.text_color,
    });

    let count_text = format!("Players: {}", game_state.players.len());
    ctx.push_cmd(DrawCmd::Text {
        text: count_text,
        pos: vec2(10.0, 20.0 + ctx.font_size * 1.5),
        font_size: ctx.font_size,
        color: ctx.theme.text_color,
    });

    // Pause button in HUD
    let sw = screen_width();
    let pause_size = vec2(60.0, 30.0);
    let pause_area = Rect::new(
        sw - pause_size.x - 10.0, // 10px from right
        10.0,                     // 10px from top
        pause_size.x,
        pause_size.y,
    );
    if Button::new("Pause").ui(ctx, pause_area) == UiResponse::Clicked {
        state.push(Screen::PauseMenu);
    }
}

/// Pause menu: allows resuming or returning to main menu
pub fn pause_menu(ctx: &mut UiContext, state: &mut UiState) {
    let area = Rect::new(0.0, 0.0, screen_width(), screen_height());
    let mut menu = VBox::new(20.0, 10.0);
    menu.begin(ctx, area);

    let paused_area = menu.item(ctx, vec2(200.0, 50.0));
    Label::new("Paused").ui(ctx, paused_area);

    let resume_area = menu.item(ctx, vec2(200.0, 50.0));
    if Button::new("Resume").ui(ctx, resume_area) == UiResponse::Clicked {
        state.pop();
    }

    let settings_area = menu.item(ctx, vec2(200.0, 50.0));
    if Button::new("Settings").ui(ctx, settings_area) == UiResponse::Clicked {
        state.push(Screen::Settings);
    }

    let main_menu_area = menu.item(ctx, vec2(200.0, 50.0));
    if Button::new("Main Menu").ui(ctx, main_menu_area) == UiResponse::Clicked {
        state.reset(Screen::MainMenu);
    }

    menu.end(ctx);
}

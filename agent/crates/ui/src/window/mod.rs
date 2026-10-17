use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use native_windows_gui as nwg;

use controls::build;
use dpi::{bind_dpi_change, read_from_system, scaled, UI_FONT_HEIGHT};
use layout::fit_window;
use paint::{bind_theme, hwnd_of, Theme};
use palette::{current_palette, system_prefers_dark};
use status::{
    announce_change, show_agent_token_state, show_mode_labels, show_panel_labels, show_token_state,
    StatusState,
};

use crate::AppHooks;

mod brushes;
mod controls;
mod dpi;
mod events;
mod layout;
mod measure;
mod paint;
mod palette;
mod status;

pub fn run(hooks: AppHooks) -> Result<(), String> {
    nwg::init().map_err(|error| format!("не удалось инициализировать интерфейс: {error}"))?;

    read_from_system();

    let mut ui_font = nwg::Font::default();
    nwg::Font::builder()
        .family("Segoe UI")
        .size_absolute(scaled(UI_FONT_HEIGHT as i32) as u32)
        .build(&mut ui_font)
        .map_err(|error| format!("не удалось выбрать шрифт: {error}"))?;
    nwg::Font::set_global_default(Some(ui_font));

    let dark = system_prefers_dark();
    let palette = current_palette();
    let palette_cell = Rc::new(std::cell::Cell::new(palette));

    let (ui, notice) = build(dark, palette, &palette_cell)?;
    let ui = Rc::new(ui);

    let active: Rc<std::cell::Cell<Option<bool>>> = Rc::new(std::cell::Cell::new(None));

    let theme = Theme {
        palette: Rc::clone(&palette_cell),
        muted: hwnd_of(ui.disclaimer.handle),
        separator: hwnd_of(ui.separator.handle),
        probe: hwnd_of(ui.probe_button.handle),
        watch: hwnd_of(ui.watch_button.handle),
        active: Rc::clone(&active),
    };
    let color_handler = bind_theme(&ui.window, theme);

    let dpi_handler = bind_dpi_change(&ui);

    let state = Rc::new(RefCell::new(StatusState::default()));
    let stored_token = Rc::new(RefCell::new((hooks.load_read_token)()));

    let sync = (hooks.load_sync)();
    ui.server_input.set_text(&sync.base_url);
    show_agent_token_state(&ui, sync.token_saved);
    show_token_state(&ui, !stored_token.borrow().is_empty());
    ui.panels.settings_open.set(!sync.token_saved);
    show_panel_labels(&ui);
    show_mode_labels(&ui, None);
    ui.status.set_text(&state.borrow().line());
    announce_change(ui.status.handle);
    ui.card
        .set_text(&crate::card::card_text(&crate::CommanderLines::default()));
    fit_window(&ui);

    let hooks = Arc::new(hooks);
    let handler = events::bind(&ui, &notice, &hooks, &state, &stored_token, &active);

    nwg::dispatch_thread_events();
    nwg::unbind_event_handler(&handler);
    if let Some(handler) = color_handler {
        nwg::unbind_raw_event_handler(&handler).ok();
    }
    if let Some(handler) = dpi_handler {
        nwg::unbind_raw_event_handler(&handler).ok();
    }
    for handler in &ui.label_hooks {
        nwg::unbind_raw_event_handler(handler).ok();
    }

    Ok(())
}

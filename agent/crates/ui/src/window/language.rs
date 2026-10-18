use native_windows_gui as nwg;

use super::controls::Ui;
use super::layout::fit_window;
use super::paint::classic_look;
use super::status::{append, show_language, StatusState};
use crate::text::{Language, ENGLISH, RUSSIAN};
use crate::AppHooks;

pub(super) fn build_radios(
    window: &nwg::Window,
) -> Result<(nwg::RadioButton, nwg::RadioButton), String> {
    let mut english = nwg::RadioButton::default();
    let mut russian = nwg::RadioButton::default();
    for (target, text, group) in [
        (&mut english, ENGLISH, nwg::RadioButtonFlags::GROUP),
        (&mut russian, RUSSIAN, nwg::RadioButtonFlags::empty()),
    ] {
        nwg::RadioButton::builder()
            .text(text)
            .flags(nwg::RadioButtonFlags::VISIBLE | nwg::RadioButtonFlags::TAB_STOP | group)
            .parent(window)
            .build(target)
            .map_err(|error| error.to_string())?;
        classic_look(target.handle);
    }
    Ok((english, russian))
}

pub(super) fn choose(
    ui: &Ui,
    hooks: &AppHooks,
    handle: nwg::ControlHandle,
    active: Option<bool>,
    state: &StatusState,
) {
    let chosen = if handle == ui.russian_radio.handle {
        Language::Russian
    } else {
        Language::English
    };
    if chosen != ui.language.get() {
        match (hooks.save_language)(chosen) {
            Ok(()) => ui.language.set(chosen),
            Err(reason) => append(ui, &format!("{}: {reason}", ui.texts().language_not_saved)),
        }
    }
    show_language(ui, active, state);
    fit_window(ui);
}

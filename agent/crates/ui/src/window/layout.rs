use super::controls::Ui;
use super::dpi::{logical, move_control, scaled};
use super::measure::{line_width, wrapped_height};
use super::status::TOGGLING_LABELS;

const MARGIN: i32 = 12;
const ROW: i32 = 28;
const BUTTON: i32 = 34;
const GAP: i32 = 10;
const STACK: i32 = 6;
const BUTTON_PADDING: i32 = 32;
const SEPARATOR: i32 = 1;
const OUTPUT_HEIGHT: i32 = 280;

fn natural_width(ui: &Ui) -> i32 {
    let font_of = ui.watch_button.handle;
    let current = ui.buttons().map(|button| button.text());
    let widest = current
        .iter()
        .map(String::as_str)
        .chain(TOGGLING_LABELS)
        .map(|label| line_width(font_of, label))
        .max()
        .unwrap_or(0);

    widest + BUTTON_PADDING + MARGIN * 2
}

fn layout_width(ui: &Ui) -> i32 {
    (logical(ui.window.size().0) as i32).max(natural_width(ui))
}

pub(super) fn fit_window(ui: &Ui) {
    let width = layout_width(ui);
    let height = arrange(ui, width - MARGIN * 2, None);
    resize(ui, width, height);
}

pub(super) fn fit_height(ui: &Ui) {
    let width = layout_width(ui);
    let required = arrange(ui, width - MARGIN * 2, None);
    let current = logical(ui.window.size().1) as i32;
    let height = if ui.panels.output_open.get() {
        current.max(required)
    } else {
        required
    };
    if height != current {
        resize(ui, width, height);
    } else {
        layout(ui);
    }
}

fn resize(ui: &Ui, width: i32, height: i32) {
    ui.window
        .set_size(scaled(width) as u32, scaled(height) as u32);
    layout(ui);
}

fn apply_visibility(ui: &Ui) {
    let panels = &ui.panels;
    let settings = panels.settings_open.get();
    let agent_stored = panels.agent_token_stored.get();
    let read_stored = panels.read_token_stored.get();

    for (control, visible) in [
        (ui.server_label.handle, settings),
        (ui.server_input.handle, settings),
        (ui.agent_token_label.handle, settings),
        (ui.agent_token_input.handle, settings && !agent_stored),
        (ui.save_sync_button.handle, settings && !agent_stored),
        (ui.edit_agent_token_button.handle, settings && agent_stored),
        (ui.token_label.handle, settings),
        (ui.token_input.handle, settings && !read_stored),
        (ui.save_button.handle, settings && !read_stored),
        (ui.copy_button.handle, settings && read_stored),
        (ui.reset_button.handle, settings && read_stored),
        (ui.dashboard_button.handle, settings),
        (ui.logs_button.handle, settings),
        (ui.logs_dir_button.handle, settings),
        (ui.resync_button.handle, settings),
        (ui.output.handle, panels.output_open.get()),
    ] {
        set_visible(control, visible);
    }
}

fn set_visible(handle: native_windows_gui::ControlHandle, visible: bool) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE, SW_SHOW};

    if let Some(hwnd) = handle.hwnd() {
        unsafe {
            ShowWindow(hwnd as _, if visible { SW_SHOW } else { SW_HIDE });
        }
    }
}

pub(super) fn layout(ui: &Ui) {
    apply_visibility(ui);

    let inner = layout_width(ui) - MARGIN * 2;
    let height = logical(ui.window.size().1) as i32;
    arrange(ui, inner, Some(height));
}

struct Column {
    inner: i32,
    y: i32,
    place: bool,
}

impl Column {
    fn put(&mut self, handle: native_windows_gui::ControlHandle, height: i32, after: i32) {
        if self.place {
            move_control(handle, MARGIN, self.y, self.inner as u32, height as u32);
        }
        self.y += height + after;
    }

    fn text(&mut self, label: &native_windows_gui::Label, after: i32) {
        let height = wrapped_height(label.handle, &label.text(), self.inner);
        self.put(label.handle, height, after);
    }

    fn buttons(&mut self, handles: &[native_windows_gui::ControlHandle], height: i32) {
        for handle in handles {
            self.put(*handle, height, STACK);
        }
        self.y += GAP - STACK;
    }
}

fn arrange(ui: &Ui, inner: i32, window_height: Option<i32>) -> i32 {
    let panels = &ui.panels;
    let mut column = Column {
        inner,
        y: MARGIN,
        place: window_height.is_some(),
    };

    column.text(&ui.status, GAP);
    column.text(&ui.card, GAP);

    column.buttons(
        &[
            ui.watch_button.handle,
            ui.probe_button.handle,
            ui.output_button.handle,
            ui.settings_button.handle,
        ],
        BUTTON,
    );

    if panels.settings_open.get() {
        column.text(&ui.server_label, 2);
        column.put(ui.server_input.handle, ROW, GAP);

        column.text(&ui.agent_token_label, 2);
        if panels.agent_token_stored.get() {
            column.buttons(&[ui.edit_agent_token_button.handle], ROW);
        } else {
            column.put(ui.agent_token_input.handle, ROW, STACK);
            column.buttons(&[ui.save_sync_button.handle], ROW);
        }

        column.text(&ui.token_label, 2);
        if panels.read_token_stored.get() {
            column.buttons(&[ui.copy_button.handle, ui.reset_button.handle], ROW);
        } else {
            column.put(ui.token_input.handle, ROW, STACK);
            column.buttons(&[ui.save_button.handle], ROW);
        }

        column.buttons(
            &[
                ui.dashboard_button.handle,
                ui.logs_button.handle,
                ui.logs_dir_button.handle,
                ui.resync_button.handle,
            ],
            ROW,
        );
    }

    let output_top = column.y;
    let output_min = if panels.output_open.get() {
        OUTPUT_HEIGHT + GAP
    } else {
        0
    };
    let disclaimer = wrapped_height(ui.disclaimer.handle, &ui.disclaimer.text(), inner);
    let required = output_top + output_min + disclaimer + MARGIN;

    if let Some(height) = window_height {
        let bottom = height.max(required);
        let disclaimer_top = bottom - MARGIN - disclaimer;
        move_control(
            ui.separator.handle,
            MARGIN,
            disclaimer_top - GAP / 2,
            inner as u32,
            SEPARATOR as u32,
        );
        move_control(
            ui.disclaimer.handle,
            MARGIN,
            disclaimer_top,
            inner as u32,
            disclaimer as u32,
        );
        if panels.output_open.get() {
            move_control(
                ui.output.handle,
                MARGIN,
                output_top,
                inner as u32,
                (disclaimer_top - GAP - output_top).max(60) as u32,
            );
        }
    }

    required
}

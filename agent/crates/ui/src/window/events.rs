use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use native_windows_gui as nwg;

use super::controls::Ui;
use super::layout::{fit_height, fit_window, layout};
use super::paint::repaint_modes;
use super::status::{
    announce_change, append, show_agent_token_state, show_mode_labels, show_panel_labels,
    show_token_state, StatusState,
};
use crate::{AppHooks, Report};

pub(super) fn bind(
    ui: &Rc<Ui>,
    notice: &nwg::Notice,
    hooks: &Arc<AppHooks>,
    state: &Rc<RefCell<StatusState>>,
    stored_token: &Rc<RefCell<String>>,
    active: &Rc<Cell<Option<bool>>>,
) -> nwg::EventHandler {
    let window_handle = ui.window.handle;

    let pending: Arc<Mutex<Vec<Report>>> = Arc::new(Mutex::new(Vec::new()));
    let running = Arc::new(AtomicBool::new(false));
    let worker: Rc<RefCell<Option<std::thread::JoinHandle<()>>>> = Rc::new(RefCell::new(None));

    {
        let ui = Rc::clone(ui);
        let state = Rc::clone(state);
        let stored_token = Rc::clone(stored_token);
        let pending = Arc::clone(&pending);
        let running = Arc::clone(&running);
        let hooks = Arc::clone(hooks);
        let worker = Rc::clone(&worker);
        let active = Rc::clone(active);
        let notice_sender = notice.sender();

        nwg::full_bind_event_handler(&window_handle, move |event, _data, handle| {
            use nwg::Event as E;

            match event {
                E::OnWindowClose => {
                    stop_worker(&running, &worker);
                    nwg::stop_thread_dispatch();
                }
                E::OnResize | E::OnWindowMaximize => layout(&ui),
                E::OnResizeEnd => fit_height(&ui),
                E::OnButtonClick if handle == ui.save_sync_button.handle => {
                    let token = ui.agent_token_input.text();

                    match (hooks.save_sync)(token.trim()) {
                        Ok(()) => {
                            ui.agent_token_input.set_text("");
                            show_agent_token_state(&ui, true);
                            fit_height(&ui);
                            append(&ui, "Настройки сохранены в профиле пользователя.");
                        }
                        Err(reason) => {
                            append(&ui, &format!("Настройки не сохранены: {reason}"));
                        }
                    }
                }
                E::OnButtonClick if handle == ui.save_button.handle => {
                    let token = ui.token_input.text();
                    if token.trim().is_empty() {
                        append(&ui, "Ключ пуст: вставьте его в поле и нажмите «Сохранить».");
                        return;
                    }

                    match (hooks.save_read_token)(token.trim()) {
                        Ok(()) => {
                            *stored_token.borrow_mut() = token.trim().to_owned();
                            ui.token_input.set_text("");
                            show_token_state(&ui, true);
                            fit_height(&ui);
                            append(
                                &ui,
                                "Ключ сохранён: он зашифрован средствами Windows и привязан к вашей учётной записи.",
                            );
                        }
                        Err(reason) => append(&ui, &format!("Ключ не сохранён: {reason}")),
                    }
                }
                E::OnButtonClick if handle == ui.copy_button.handle => {
                    let token = stored_token.borrow().clone();
                    if token.is_empty() {
                        append(&ui, "Копировать нечего: ключ ещё не сохранён.");
                    } else {
                        nwg::Clipboard::set_data_text(&ui.window.handle, &token);
                        append(&ui, "Ключ скопирован в буфер обмена.");
                    }
                }
                E::OnButtonClick if handle == ui.reset_button.handle => {
                    match (hooks.clear_read_token)() {
                        Ok(()) => {
                            stored_token.borrow_mut().clear();
                            ui.token_input.set_text("");
                            show_token_state(&ui, false);
                            fit_height(&ui);
                            append(&ui, "Ключ забыт. Вставьте новый, когда понадобится.");
                        }
                        Err(reason) => append(&ui, &format!("Ключ не сброшен: {reason}")),
                    }
                }
                E::OnButtonClick if handle == ui.dashboard_button.handle => {
                    if let Err(reason) = (hooks.open_dashboard)() {
                        append(&ui, &format!("Не удалось открыть сайт: {reason}"));
                    }
                }
                E::OnButtonClick if handle == ui.logs_button.handle => {
                    if let Err(reason) = (hooks.open_logs)() {
                        append(&ui, &format!("Не удалось открыть лог: {reason}"));
                    }
                }
                E::OnButtonClick if handle == ui.logs_dir_button.handle => {
                    if let Err(reason) = (hooks.open_logs_dir)() {
                        append(&ui, &format!("Не удалось открыть папку с логами: {reason}"));
                    }
                }
                E::OnButtonClick if handle == ui.output_button.handle => {
                    let open = &ui.panels.output_open;
                    open.set(!open.get());
                    show_panel_labels(&ui);
                    fit_window(&ui);
                }
                E::OnButtonClick if handle == ui.settings_button.handle => {
                    let open = &ui.panels.settings_open;
                    open.set(!open.get());
                    show_panel_labels(&ui);
                    fit_window(&ui);
                }
                E::OnButtonClick if handle == ui.edit_agent_token_button.handle => {
                    show_agent_token_state(&ui, false);
                    fit_height(&ui);
                    ui.agent_token_input.set_focus();
                }
                E::OnButtonClick
                    if handle == ui.probe_button.handle
                        || handle == ui.watch_button.handle
                        || handle == ui.resync_button.handle =>
                {
                    let is_probe = handle == ui.probe_button.handle;
                    let is_resync = handle == ui.resync_button.handle;

                    if is_resync && running.load(Ordering::SeqCst) {
                        append(
                            &ui,
                            "Сначала остановите работающий режим — потом «Отправить всё заново».",
                        );
                        return;
                    }

                    if active.get() == Some(is_probe) {
                        running.store(false, Ordering::SeqCst);
                        active.set(None);
                        repaint_modes(&ui);
                        show_mode_labels(&ui, active.get());
                        append(&ui, "Остановлено.");
                        return;
                    }

                    if running.load(Ordering::SeqCst) {
                        append(&ui, "Один режим уже работает — сначала остановите его.");
                        return;
                    }

                    active.set(Some(is_probe));
                    repaint_modes(&ui);
                    show_mode_labels(&ui, active.get());

                    ui.output.set_text("");
                    running.store(true, Ordering::SeqCst);

                    let pending = Arc::clone(&pending);
                    let running_flag = Arc::clone(&running);
                    let hooks = Arc::clone(&hooks);
                    let sender = notice_sender;

                    *worker.borrow_mut() = Some(std::thread::spawn(move || {
                        let sink: crate::Sink = Arc::new(move |report: Report| {
                            if let Ok(mut queue) = pending.lock() {
                                queue.push(report);
                            }
                            sender.notice();
                        });

                        if is_probe {
                            (hooks.probe)(Arc::clone(&sink));
                        } else if is_resync {
                            (hooks.resync)(Arc::clone(&sink), &running_flag);
                        } else {
                            (hooks.watch)(Arc::clone(&sink), &running_flag);
                        }

                        running_flag.store(false, Ordering::SeqCst);
                        sink(Report::Line("--- режим завершён".to_owned()));
                    }));
                }
                E::OnNotice => {
                    let reports: Vec<Report> = match pending.lock() {
                        Ok(mut queue) => queue.drain(..).collect(),
                        Err(_) => Vec::new(),
                    };

                    let mut status_changed = false;
                    let mut card_changed = false;
                    for report in reports {
                        match report {
                            Report::Line(line) => append(&ui, &line),
                            Report::Game(found) => {
                                state.borrow_mut().game = Some(found);
                                status_changed = true;
                            }
                            Report::Connection(ok) => {
                                state.borrow_mut().connection = Some(ok);
                                status_changed = true;
                            }
                            Report::LastSync(at) => {
                                state.borrow_mut().last_sync = Some(at);
                                status_changed = true;
                            }
                            Report::Commander(lines) => {
                                ui.card.set_text(&crate::card::card_text(&lines));
                                announce_change(ui.card.handle);
                                card_changed = true;
                            }
                        }
                    }

                    if status_changed {
                        ui.status.set_text(&state.borrow().line());
                        announce_change(ui.status.handle);
                    }
                    if status_changed || card_changed {
                        fit_height(&ui);
                    }

                    if !running.load(Ordering::SeqCst) && active.get().is_some() {
                        active.set(None);
                        repaint_modes(&ui);
                        show_mode_labels(&ui, active.get());
                    }
                }
                _ => {}
            }
        })
    }
}

fn stop_worker(
    running: &Arc<AtomicBool>,
    worker: &Rc<RefCell<Option<std::thread::JoinHandle<()>>>>,
) {
    running.store(false, Ordering::SeqCst);
    if let Some(handle) = worker.borrow_mut().take() {
        let _ = handle.join();
    }
}

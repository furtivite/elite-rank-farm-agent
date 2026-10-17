use std::cell::RefCell;

pub type Sink = Box<dyn Fn(String)>;

pub type StatusSink = Box<dyn Fn(Status)>;

#[derive(Debug, Clone)]
pub enum Status {
    Game(bool),
    Connection(bool),
    LastSync(String),
    Commander(agent_core::CommanderCard),
}

thread_local! {
    static SINK: RefCell<Option<Sink>> = const { RefCell::new(None) };
    static STATUS: RefCell<Option<StatusSink>> = const { RefCell::new(None) };
}

pub fn install(sink: Sink) {
    SINK.with(|current| {
        *current.borrow_mut() = Some(sink);
    });
}

pub fn install_status(sink: StatusSink) {
    STATUS.with(|current| {
        *current.borrow_mut() = Some(sink);
    });
}

pub fn status(update: Status) {
    STATUS.with(|current| {
        if let Some(sink) = current.borrow().as_ref() {
            sink(update);
        }
    });
}

pub fn emit(line: String) {
    crate::log::append(&line);

    SINK.with(|current| match current.borrow().as_ref() {
        Some(sink) => sink(line),
        None => println!("{line}"),
    });
}

#[macro_export]
macro_rules! say {
    () => { $crate::sink::emit(String::new()) };
    ($($arg:tt)*) => { $crate::sink::emit(format!($($arg)*)) };
}

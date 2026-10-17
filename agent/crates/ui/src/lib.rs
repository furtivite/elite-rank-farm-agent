use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Report {
    Line(String),
    Game(bool),
    Connection(bool),
    LastSync(String),
    Commander(CommanderLines),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommanderLines {
    pub cmdr: Option<String>,
    pub ship: Option<String>,
    pub suit: Option<String>,
    pub system: Option<String>,
    pub station: Option<String>,
}

pub type Sink = Arc<dyn Fn(Report) + Send + Sync>;

pub type LoadToken = Box<dyn Fn() -> String + Send + Sync>;

pub type SaveToken = Box<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

pub type RunOnce = Arc<dyn Fn(Sink) + Send + Sync>;

pub type RunUntilStopped = Arc<dyn Fn(Sink, &AtomicBool) + Send + Sync>;

pub type Action = Box<dyn Fn() -> Result<(), String> + Send + Sync>;

#[derive(Debug, Default, Clone)]
pub struct SyncSettings {
    pub base_url: String,
    pub token_saved: bool,
}

pub type LoadSync = Box<dyn Fn() -> SyncSettings + Send + Sync>;

pub type SaveSync = Box<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

pub struct AppHooks {
    pub load_sync: LoadSync,
    pub save_sync: SaveSync,
    pub load_read_token: LoadToken,
    pub save_read_token: SaveToken,
    pub clear_read_token: Action,
    pub open_dashboard: Action,
    pub open_logs: Action,
    pub open_logs_dir: Action,
    pub probe: RunOnce,
    pub watch: RunUntilStopped,
    pub resync: RunUntilStopped,
}

pub fn window_title() -> String {
    format!(
        "Elite Rank Farm Navigator — агент {}",
        env!("CARGO_PKG_VERSION")
    )
}

#[cfg_attr(not(windows), allow(dead_code))]
mod card;

#[cfg_attr(not(windows), allow(dead_code))]
mod theme;

#[cfg_attr(not(windows), allow(dead_code))]
mod icon;

#[cfg(windows)]
mod window;

#[cfg(windows)]
pub fn run(hooks: AppHooks) -> Result<(), String> {
    window::run(hooks)
}

#[cfg(not(windows))]
pub fn run(_hooks: AppHooks) -> Result<(), String> {
    Err("окно агента доступно только в Windows".to_owned())
}

#[cfg(test)]
mod tests {
    use super::window_title;

    #[test]
    fn заголовок_окна_называет_версию_сборки() {
        let title = window_title();

        assert!(title.starts_with("Elite Rank Farm Navigator"), "{title}");
        assert!(title.ends_with(env!("CARGO_PKG_VERSION")), "{title}");
        assert!(!env!("CARGO_PKG_VERSION").is_empty());
    }
}

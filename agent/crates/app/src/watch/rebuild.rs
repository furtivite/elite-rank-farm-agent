use std::collections::BTreeSet;

use agent_sync::{RebuildAccepted, StateClient, SyncConfig};

use crate::{say, say_tr};

pub fn rebuild_only() -> Result<(), String> {
    let config = SyncConfig::load().map_err(|error| error.to_string())?;
    let client = StateClient::new(&config);

    say_tr!("Rebuild at {}", "Пересборка на {}", config.rebuild_url());
    match client.request_rebuild() {
        Ok(RebuildAccepted { rebuilt, warnings }) => {
            if rebuilt.is_empty() {
                say_tr!(
                    "Nothing to rebuild: the data has not changed since the last build.",
                    "Пересобирать нечего: данные не менялись с прошлой сборки."
                );
            }
            for region in &rebuilt {
                say_tr!(
                    "{} rebuilt, revision {}, systems {}, stations {}",
                    "{} пересобран, ревизия {}, систем {}, станций {}",
                    region.region,
                    region.revision,
                    region.systems,
                    region.stations,
                );
            }
            for warning in &warnings {
                say_tr!("warning: {warning}", "предупреждение: {warning}");
            }
            Ok(())
        }
        Err(error) => Err(format!("{error}")),
    }
}

pub(super) fn report_rebuild(client: &StateClient, said: &mut BTreeSet<String>) {
    match client.request_rebuild() {
        Ok(RebuildAccepted { rebuilt, warnings }) => {
            if rebuilt.is_empty() {
                say_tr!(
                    "  bootstrap: nothing to rebuild, the data has not changed",
                    "  bootstrap: пересобирать нечего, данные не менялись"
                );
            } else {
                for region in &rebuilt {
                    say_tr!(
                        "  bootstrap: {} rebuilt, revision {}, systems {}, stations {}",
                        "  bootstrap: {} пересобран, ревизия {}, систем {}, станций {}",
                        region.region,
                        region.revision,
                        region.systems,
                        region.stations,
                    );
                }
            }
            for warning in &warnings {
                if said.insert(warning.clone()) {
                    say!("  bootstrap: {warning}");
                }
            }
        }
        Err(error) => say_tr!(
            "  bootstrap not rebuilt: {error}",
            "  bootstrap не пересобран: {error}"
        ),
    }
}

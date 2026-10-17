use std::collections::BTreeSet;

use agent_sync::{RebuildAccepted, StateClient, SyncConfig};

use crate::say;

pub fn rebuild_only() -> Result<(), String> {
    let config = SyncConfig::load().map_err(|error| error.to_string())?;
    let client = StateClient::new(&config);

    say!("Пересборка на {}", config.rebuild_url());
    match client.request_rebuild() {
        Ok(RebuildAccepted { rebuilt, warnings }) => {
            if rebuilt.is_empty() {
                say!("Пересобирать нечего: данные не менялись с прошлой сборки.");
            }
            for region in &rebuilt {
                say!(
                    "{} пересобран, ревизия {}, систем {}, станций {}",
                    region.region,
                    region.revision,
                    region.systems,
                    region.stations,
                );
            }
            for warning in &warnings {
                say!("предупреждение: {warning}");
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
                say!("  bootstrap: пересобирать нечего, данные не менялись");
            } else {
                for region in &rebuilt {
                    say!(
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
        Err(error) => say!("  bootstrap не пересобран: {error}"),
    }
}

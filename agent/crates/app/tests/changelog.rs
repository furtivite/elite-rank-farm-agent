use std::path::PathBuf;

fn changelog() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../CHANGELOG.md");
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn current_section(text: &str) -> Vec<&str> {
    let version = env!("CARGO_PKG_VERSION");
    let heading = format!("## {version} ");
    let start = text
        .lines()
        .position(|line| line.starts_with(&heading))
        .unwrap_or_else(|| panic!("в agent/CHANGELOG.md нет раздела «## {version} — <дата>»"));
    text.lines()
        .skip(start + 1)
        .take_while(|line| !line.starts_with("## "))
        .collect()
}

#[test]
fn current_version_is_described_in_english_and_in_russian() {
    let text = changelog();
    let section = current_section(&text);

    for language in ["### English", "### Русский"] {
        let start = section
            .iter()
            .position(|line| line.trim() == language)
            .unwrap_or_else(|| panic!("в разделе {} нет «{language}»", env!("CARGO_PKG_VERSION")));
        let filled = section
            .iter()
            .skip(start + 1)
            .take_while(|line| !line.starts_with("### "))
            .any(|line| !line.trim().is_empty());
        assert!(
            filled,
            "часть «{language}» раздела {} пуста",
            env!("CARGO_PKG_VERSION")
        );
    }
}

#[test]
fn newest_section_is_the_current_version() {
    let text = changelog();
    let first = text
        .lines()
        .find(|line| line.starts_with("## "))
        .expect("в CHANGELOG.md нет ни одного раздела");

    assert!(
        first.starts_with(&format!("## {} ", env!("CARGO_PKG_VERSION"))),
        "первый раздел — «{first}», а версия агента {}",
        env!("CARGO_PKG_VERSION")
    );
}

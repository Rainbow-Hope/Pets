use std::fs;
use std::path::{Path, PathBuf};

fn assets() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("package-assets")
}

#[test]
fn comparison_document_has_the_required_order_and_all_editions() {
    let text = fs::read_to_string(assets().join("DIFERENCAS-ENTRE-EDICOES.txt"))
        .expect("comparison document");

    assert!(text.starts_with("Normal > Micro > Nano > Pico"));
    let index = text.find("ÍNDICE").expect("index");
    let table = text.find("COMPARAÇÃO RÁPIDA").expect("comparison table");
    let normal = text.find("NORMAL —").expect("normal section");
    let micro = text.find("MICRO —").expect("micro section");
    let nano = text.find("NANO —").expect("nano section");
    let pico = text.find("PICO —").expect("pico section");
    let measurements = text.find("MEDIÇÕES").expect("measurements");

    assert!(index < table);
    assert!(table < normal);
    assert!(normal < micro);
    assert!(micro < nano);
    assert!(nano < pico);
    assert!(pico < measurements);
}

#[test]
fn readme_states_that_packages_are_autonomous_not_installers() {
    let text = fs::read_to_string(assets().join("LEIA-ME.txt")).expect("readme");
    let lowercase = text.to_lowercase();

    assert!(lowercase.contains("não é instalador"));
    assert!(lowercase.contains("não precisa do códex"));
    assert!(lowercase.contains("sem internet"));
    assert!(lowercase.contains("config.json"));
    assert!(lowercase.contains("pasta pets"));
    for edition in ["Normal", "Micro", "Nano", "Pico"] {
        assert!(text.contains(edition), "missing {edition}");
    }
}

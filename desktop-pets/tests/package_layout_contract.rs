use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repository root")
        .to_path_buf()
}

#[test]
fn all_four_portable_packages_have_executable_config_docs_and_rainbow_hope() {
    let output = repository_root().join("Executar fora do Códex");
    let packages = [
        ("Normal", "DesktopPets.exe"),
        ("Leves/Micro", "DesktopPetsMicro.exe"),
        ("Leves/Nano", "DesktopPetsNano.exe"),
        ("Leves/Pico", "DesktopPetsPico.exe"),
    ];

    for (relative, executable) in packages {
        let package = output.join(relative);
        for required in [
            executable,
            "config.json",
            "LEIA-ME.txt",
            "DIFERENCAS-ENTRE-EDICOES.txt",
            "pets/rainbow-hope/pet.json",
            "pets/rainbow-hope/spritesheet.webp",
        ] {
            assert!(
                package.join(required).is_file(),
                "missing {}",
                package.join(required).display()
            );
        }
    }

    assert!(output.join("LEIA-ME.txt").is_file());
    assert!(output.join("DIFERENCAS-ENTRE-EDICOES.txt").is_file());
}

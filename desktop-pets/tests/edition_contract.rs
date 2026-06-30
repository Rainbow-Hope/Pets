use desktop_pets::edition::Edition;

#[test]
fn normal_and_light_profiles_match_the_approved_capability_matrix() {
    let normal = Edition::Normal.profile();
    let micro = Edition::Micro.profile();
    let nano = Edition::Nano.profile();
    let pico = Edition::Pico.profile();

    assert_eq!(normal.max_windows, None);
    assert_eq!(micro.max_windows, Some(4));
    assert_eq!(nano.max_windows, Some(1));
    assert_eq!(pico.max_windows, Some(1));

    assert!(normal.can_import);
    assert!(micro.can_import);
    assert!(!nano.can_import);
    assert!(!pico.can_import);

    assert!(normal.can_move);
    assert!(micro.can_move);
    assert!(nano.can_move);
    assert!(!pico.can_move);

    assert!(normal.can_startup);
    assert!(micro.can_startup);
    assert!(!nano.can_startup);
    assert!(!pico.can_startup);

    assert_eq!(normal.frame_interval_ms, 100);
    assert_eq!(micro.frame_interval_ms, 200);
    assert_eq!(nano.metric_interval_ms, 5_000);
    assert_eq!(pico.metric_interval_ms, 8_000);
}

#[test]
fn each_edition_has_an_independent_executable_and_coordinator_identity() {
    let editions = [
        Edition::Normal,
        Edition::Micro,
        Edition::Nano,
        Edition::Pico,
    ];
    let executable_names: Vec<_> = editions
        .iter()
        .map(|edition| edition.profile().executable_name)
        .collect();
    let coordinator_suffixes: Vec<_> = editions
        .iter()
        .map(|edition| edition.profile().coordinator_suffix)
        .collect();

    assert_eq!(
        executable_names,
        [
            "DesktopPets.exe",
            "DesktopPetsMicro.exe",
            "DesktopPetsNano.exe",
            "DesktopPetsPico.exe",
        ]
    );
    assert_eq!(coordinator_suffixes, ["normal", "micro", "nano", "pico"]);
}

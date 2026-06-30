use desktop_pets::pet::{
    ATLAS_HEIGHT, ATLAS_WIDTH, Atlas, AtlasGeometry, CELL_HEIGHT, CELL_WIDTH, PetError, PetManifest,
    PetState,
};
use std::path::Path;

#[test]
fn codex_atlas_geometry_and_frame_counts_are_fixed() {
    assert_eq!((ATLAS_WIDTH, ATLAS_HEIGHT), (1536, 1872));
    assert_eq!((CELL_WIDTH, CELL_HEIGHT), (192, 208));

    let expected = [
        (PetState::Idle, 0, 6),
        (PetState::RunningRight, 1, 8),
        (PetState::RunningLeft, 2, 8),
        (PetState::Waving, 3, 4),
        (PetState::Jumping, 4, 5),
        (PetState::Failed, 5, 8),
        (PetState::Waiting, 6, 6),
        (PetState::Running, 7, 6),
        (PetState::Review, 8, 6),
    ];

    for (state, row, frames) in expected {
        assert_eq!(state.row(), row);
        assert_eq!(state.frame_count(), frames);
    }
}

#[test]
fn light_editions_accept_only_approved_proportional_atlas_sizes() {
    let approved = [
        (1536, 1872, 192, 208),
        (1152, 1404, 144, 156),
        (768, 936, 96, 104),
        (384, 468, 48, 52),
    ];
    for (width, height, cell_width, cell_height) in approved {
        let geometry = AtlasGeometry::from_dimensions(width, height).expect("approved geometry");
        assert_eq!((geometry.cell_width, geometry.cell_height), (cell_width, cell_height));
    }

    assert!(AtlasGeometry::from_dimensions(1000, 1000).is_none());
    assert!(AtlasGeometry::from_dimensions(192, 208).is_none());
}

#[test]
fn packaged_light_rainbow_hope_atlases_decode_with_expected_cells() {
    let assets = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("package-assets")
        .join("light-atlases");
    for (name, cell_width, cell_height) in [
        ("micro.webp", 144, 156),
        ("nano.webp", 96, 104),
        ("pico.webp", 48, 52),
    ] {
        let atlas = Atlas::load(&assets.join(name)).expect("light atlas");
        let frame = atlas.frame(PetState::Idle, 0);
        assert_eq!((frame.width, frame.height), (cell_width, cell_height));
    }
}

#[test]
fn rainbow_hope_manifest_and_atlas_load_from_repository_assets() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repository root");
    let pet_dir = repo.join("pets").join("rainbow-hope");
    let manifest = PetManifest::load(&pet_dir.join("pet.json")).expect("manifest");
    let atlas = Atlas::load(&pet_dir.join(&manifest.spritesheet_path)).expect("atlas");

    assert_eq!(manifest.id, "rainbow-hope");
    assert_eq!(manifest.display_name, "Rainbow Hope");
    for state in PetState::ALL {
        assert_eq!(atlas.frames(state).len(), state.frame_count());
        let frame = atlas.frame(state, state.frame_count());
        assert_eq!(frame.width, CELL_WIDTH);
        assert_eq!(frame.height, CELL_HEIGHT);
        assert_eq!(frame.alpha_at(0, 0), frame.premultiplied_bgra[3]);
    }
}

#[test]
fn manifest_rejects_parent_traversal_and_invalid_identifiers() {
    let traversal = r#"{
        "id": "rainbow-hope",
        "displayName": "Rainbow Hope",
        "spritesheetPath": "../outside.webp"
    }"#;
    let invalid_id = r#"{
        "id": "Rainbow Hope!",
        "displayName": "Rainbow Hope",
        "spritesheetPath": "spritesheet.webp"
    }"#;

    assert!(matches!(
        PetManifest::from_json(traversal),
        Err(PetError::UnsafeSpritesheetPath(_))
    ));
    assert!(matches!(
        PetManifest::from_json(invalid_id),
        Err(PetError::InvalidId(_))
    ));
}

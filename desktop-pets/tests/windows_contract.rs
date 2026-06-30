use desktop_pets::config::MovementMode;
use desktop_pets::windows::{
    command_to_movement, directory_scope_id,
    metrics::{CpuTimes, cpu_percent},
};
use std::path::Path;

#[test]
fn cpu_delta_subtracts_idle_time_from_kernel_and_user_time() {
    let previous = CpuTimes {
        idle: 100,
        kernel: 300,
        user: 200,
    };
    let current = CpuTimes {
        idle: 120,
        kernel: 350,
        user: 250,
    };

    assert!((cpu_percent(previous, current).expect("valid delta") - 80.0).abs() < 0.001);
    assert!(cpu_percent(current, previous).is_none());
}

#[test]
fn executable_directory_scope_is_stable_and_path_specific() {
    let first = directory_scope_id(Path::new(r"C:\Portable\DesktopPets"));
    let repeated = directory_scope_id(Path::new(r"C:\Portable\DesktopPets"));
    let copied = directory_scope_id(Path::new(r"D:\USB\DesktopPets"));

    assert_eq!(first, repeated);
    assert_ne!(first, copied);
    assert_eq!(first.len(), 16);
}

#[test]
fn native_menu_commands_cover_all_movement_modes() {
    let expected = [
        MovementMode::Fixed,
        MovementMode::BottomStrip,
        MovementMode::Line,
        MovementMode::WholeScreen,
        MovementMode::BetweenMonitors,
        MovementMode::SemiFixed,
    ];

    for (offset, mode) in expected.into_iter().enumerate() {
        assert_eq!(command_to_movement(200 + offset as u32), Some(mode));
    }
    assert_eq!(command_to_movement(999), None);
}

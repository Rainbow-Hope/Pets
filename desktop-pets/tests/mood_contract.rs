use desktop_pets::behavior::mood::{
    EXHAUSTED_DWELL_MS, Mood, MoodEngine, ResourceSample, combined_pressure,
};
use desktop_pets::pet::PetState;

fn sample(cpu: f32, memory: f32) -> ResourceSample {
    ResourceSample {
        cpu_percent: cpu,
        memory_percent: memory,
    }
}

#[test]
fn combined_pressure_weights_cpu_sixty_five_percent() {
    let score = combined_pressure(sample(80.0, 40.0));
    assert!((score - 0.66).abs() < 0.0001);
}

#[test]
fn sustained_absolute_overload_becomes_exhausted_after_dwell() {
    let mut engine = MoodEngine::with_seed(7);

    assert_eq!(engine.update(0, Ok(sample(96.0, 95.0))).mood, Mood::Calm);
    assert_ne!(
        engine
            .update(EXHAUSTED_DWELL_MS - 1, Ok(sample(96.0, 95.0)))
            .mood,
        Mood::Exhausted
    );
    let decision = engine.update(EXHAUSTED_DWELL_MS, Ok(sample(96.0, 95.0)));

    assert_eq!(decision.mood, Mood::Exhausted);
    assert_eq!(decision.animation, PetState::Failed);
}

#[test]
fn failed_metric_collection_falls_back_to_calm_animation() {
    let mut engine = MoodEngine::with_seed(11);

    let decision = engine.update(0, Err(()));

    assert_eq!(decision.mood, Mood::Calm);
    assert!(matches!(
        decision.animation,
        PetState::Idle | PetState::Waiting
    ));
}

#[test]
fn baseline_keeps_only_the_latest_fifteen_minutes() {
    let mut engine = MoodEngine::with_seed(13);
    engine.update(0, Ok(sample(20.0, 30.0)));
    engine.update(899_000, Ok(sample(25.0, 35.0)));
    assert_eq!(engine.baseline_len(), 2);

    engine.update(901_000, Ok(sample(25.0, 35.0)));

    assert_eq!(engine.baseline_len(), 2);
}

#[test]
fn random_events_are_deterministic_and_only_interrupt_low_pressure_moods() {
    let mut calm = MoodEngine::with_seed(17);
    calm.update(0, Ok(sample(5.0, 20.0)));
    let event_at = calm.next_random_event_at();
    let event = calm.update(event_at, Ok(sample(5.0, 20.0))).animation;
    assert!(matches!(event, PetState::Waving | PetState::Jumping));

    let mut overloaded = MoodEngine::with_seed(17);
    overloaded.update(0, Ok(sample(99.0, 99.0)));
    overloaded.update(EXHAUSTED_DWELL_MS, Ok(sample(99.0, 99.0)));
    let event_at = overloaded.next_random_event_at();
    let decision = overloaded.update(event_at, Ok(sample(99.0, 99.0)));
    assert_eq!(decision.animation, PetState::Failed);
}

use desktop_pets::behavior::movement::{
    Direction, Motion, MovementPlanner, Rect, Size, clamp_to_work_areas,
};
use desktop_pets::config::{MovementMode, Point, SemiFixedPoints};
use desktop_pets::pet::PetState;

const PET: Size = Size {
    width: 192,
    height: 208,
};
const LEFT: Rect = Rect {
    left: 0,
    top: 0,
    right: 1920,
    bottom: 1040,
};
const RIGHT: Rect = Rect {
    left: 1920,
    top: 0,
    right: 3840,
    bottom: 1040,
};

#[test]
fn fixed_mode_has_no_autonomous_destination() {
    let mut planner = MovementPlanner::with_seed(5);
    assert_eq!(
        planner.plan(
            MovementMode::Fixed,
            Point { x: 100, y: 200 },
            PET,
            &[LEFT],
            SemiFixedPoints::default(),
        ),
        None
    );
}

#[test]
fn bottom_strip_and_line_preserve_their_vertical_constraints() {
    let mut planner = MovementPlanner::with_seed(7);
    let bottom = planner
        .plan(
            MovementMode::BottomStrip,
            Point { x: 100, y: 200 },
            PET,
            &[LEFT],
            SemiFixedPoints::default(),
        )
        .expect("bottom motion");
    assert_eq!(bottom.to.y, LEFT.bottom - PET.height);

    let line = planner
        .plan(
            MovementMode::Line,
            Point { x: 100, y: 350 },
            PET,
            &[LEFT],
            SemiFixedPoints::default(),
        )
        .expect("line motion");
    assert_eq!(line.to.y, 350);
}

#[test]
fn whole_screen_stays_on_current_monitor_and_cross_monitor_can_leave_it() {
    let mut planner = MovementPlanner::with_seed(11);
    let whole = planner
        .plan(
            MovementMode::WholeScreen,
            Point { x: 100, y: 200 },
            PET,
            &[LEFT, RIGHT],
            SemiFixedPoints::default(),
        )
        .expect("whole screen");
    assert!(LEFT.contains_top_left(whole.to, PET));

    let mut crossed = false;
    for _ in 0..16 {
        let motion = planner
            .plan(
                MovementMode::BetweenMonitors,
                Point { x: 100, y: 200 },
                PET,
                &[LEFT, RIGHT],
                SemiFixedPoints::default(),
            )
            .expect("cross monitor");
        crossed |= RIGHT.contains_top_left(motion.to, PET);
    }
    assert!(crossed);
}

#[test]
fn semi_fixed_moves_to_the_other_saved_point() {
    let mut planner = MovementPlanner::with_seed(13);
    let points = SemiFixedPoints {
        a: Some(Point { x: 100, y: 100 }),
        b: Some(Point { x: 1000, y: 700 }),
    };

    let motion = planner
        .plan(
            MovementMode::SemiFixed,
            Point { x: 100, y: 100 },
            PET,
            &[LEFT],
            points,
        )
        .expect("semi-fixed motion");

    assert_eq!(motion.to, Point { x: 1000, y: 700 });
}

#[test]
fn clamping_and_direction_produce_valid_visible_motion() {
    assert_eq!(
        clamp_to_work_areas(Point { x: 4000, y: -20 }, PET, &[LEFT, RIGHT]),
        Point {
            x: RIGHT.right - PET.width,
            y: RIGHT.top,
        }
    );

    let right = Motion::new(Point { x: 10, y: 10 }, Point { x: 20, y: 50 });
    let left = Motion::new(Point { x: 20, y: 10 }, Point { x: 10, y: 50 });
    assert_eq!(right.direction, Direction::Right);
    assert_eq!(right.animation, PetState::RunningRight);
    assert_eq!(left.direction, Direction::Left);
    assert_eq!(left.animation, PetState::RunningLeft);
}

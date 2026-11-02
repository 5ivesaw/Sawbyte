use sawbyte_core::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct State {
    frames: Vec<InputFrame>,
    releases: usize,
    fail: bool,
    fail_release: bool,
}

struct TestBackend(Rc<RefCell<State>>);

impl Backend for TestBackend {
    type Error = &'static str;
    fn supports(&self, _: Capability) -> bool {
        true
    }
    fn apply(&mut self, _: u64, frame: InputFrame) -> Result<(), Self::Error> {
        let mut state = self.0.borrow_mut();
        if state.fail {
            return Err("disconnected");
        }
        state.frames.push(frame);
        Ok(())
    }
    fn release_all(&mut self) -> Result<(), Self::Error> {
        let mut state = self.0.borrow_mut();
        state.releases += 1;
        if state.fail_release {
            Err("release failed")
        } else {
            Ok(())
        }
    }
}

fn down() -> Action {
    Action::Button {
        button: Button::Forward,
        down: true,
    }
}

#[test]
fn cancelled_future_actions_never_execute_and_keys_are_released() {
    let state = Rc::new(RefCell::new(State::default()));
    let mut scheduler = Scheduler::new(TestBackend(state.clone()), Direct, 8);
    scheduler.submit(&[(0, down()), (5, down())]).unwrap();
    assert!(scheduler
        .advance(0)
        .unwrap()
        .buttons
        .pressed(Button::Forward));
    scheduler.cancel().unwrap();
    assert_eq!(state.borrow().releases, 1);
    assert_eq!(scheduler.pending(), 0);
    for tick in 1..=5 {
        assert_eq!(scheduler.advance(tick).unwrap(), InputFrame::default());
    }
}

#[test]
fn batches_are_atomic_and_queue_is_bounded() {
    let state = Rc::new(RefCell::new(State::default()));
    let mut scheduler = Scheduler::new(TestBackend(state), Direct, 2);
    assert_eq!(
        scheduler.submit(&[(1, down()), (0, down())]),
        Err(Error::OutOfOrder)
    );
    assert_eq!(scheduler.pending(), 0);
    scheduler.submit(&[(0, down()), (1, down())]).unwrap();
    assert_eq!(scheduler.submit(&[(2, down())]), Err(Error::QueueFull));
    assert_eq!(scheduler.pending(), 2);
}

#[test]
fn camera_policy_preserves_total_motion_and_direct_mode_is_exact() {
    let state = Rc::new(RefCell::new(State::default()));
    let mut scheduler = Scheduler::new(TestBackend(state.clone()), Humanised::default(), 2);
    scheduler
        .submit(&[(
            0,
            Action::Look {
                yaw: 90.0,
                pitch: -45.0,
            },
        )])
        .unwrap();
    let mut sum = (0.0, 0.0);
    for tick in 0..12 {
        let frame = scheduler.advance(tick).unwrap();
        assert!(frame.yaw_delta.hypot(frame.pitch_delta) <= 12.0 + 1e-10);
        sum.0 += frame.yaw_delta;
        sum.1 += frame.pitch_delta;
    }
    assert!((sum.0 - 90.0).abs() < 1e-10);
    assert!((sum.1 + 45.0).abs() < 1e-10);
    let mut direct = Scheduler::new(TestBackend(state), Direct, 1);
    direct
        .submit(&[(
            0,
            Action::Look {
                yaw: 90.0,
                pitch: -45.0,
            },
        )])
        .unwrap();
    assert_eq!(direct.advance(0).unwrap().yaw_delta, 90.0);
}

#[test]
fn failure_halts_and_does_not_replay_uncertain_actions() {
    let state = Rc::new(RefCell::new(State {
        fail: true,
        ..State::default()
    }));
    let mut scheduler = Scheduler::new(TestBackend(state.clone()), Direct, 4);
    scheduler.submit(&[(0, down()), (1, down())]).unwrap();
    assert!(matches!(scheduler.advance(0), Err(Error::Backend { .. })));
    assert_eq!(state.borrow().releases, 1);
    assert_eq!(scheduler.pending(), 0);
    assert_eq!(scheduler.advance(1), Err(Error::Halted));
    state.borrow_mut().fail = false;
    scheduler.cancel().unwrap();
    assert_eq!(scheduler.advance(1).unwrap(), InputFrame::default());
}

#[test]
fn clock_cannot_rewind_or_skip_edges() {
    let state = Rc::new(RefCell::new(State::default()));
    let mut scheduler = Scheduler::new(TestBackend(state), Direct, 2);
    scheduler.submit(&[(1, down())]).unwrap();
    assert_eq!(scheduler.advance(2), Err(Error::OutOfOrder));
    scheduler.advance(1).unwrap();
    assert_eq!(scheduler.advance(1), Err(Error::OutOfOrder));
    assert_eq!(scheduler.advance(3), Err(Error::OutOfOrder));
    assert_eq!(scheduler.submit(&[(1, down())]), Err(Error::OutOfOrder));
}

#[test]
fn invalid_numeric_input_is_rejected_before_admission() {
    let state = Rc::new(RefCell::new(State::default()));
    let mut scheduler = Scheduler::new(TestBackend(state), Direct, 2);
    for yaw in [f64::NAN, f64::INFINITY, 361.0] {
        assert_eq!(
            scheduler.submit(&[(0, Action::Look { yaw, pitch: 0.0 })]),
            Err(Error::InvalidAction)
        );
    }
    assert!(Humanised::new(0.0).is_none());
}

#[test]
fn custom_policy_cannot_inject_nonfinite_motion() {
    struct Broken;
    impl InputPolicy for Broken {
        fn camera_step(&mut self, _: (f64, f64)) -> (f64, f64) {
            (f64::NAN, 0.0)
        }
    }
    let state = Rc::new(RefCell::new(State::default()));
    let mut scheduler = Scheduler::new(TestBackend(state.clone()), Broken, 1);
    assert_eq!(scheduler.advance(0), Err(Error::InvalidPolicy));
    assert!(state.borrow().frames.is_empty());
    assert_eq!(state.borrow().releases, 1);
}

#[test]
fn release_failure_stays_halted_and_is_reported() {
    let state = Rc::new(RefCell::new(State {
        fail: true,
        fail_release: true,
        ..State::default()
    }));
    let mut scheduler = Scheduler::new(TestBackend(state), Direct, 1);
    assert!(matches!(
        scheduler.advance(0),
        Err(Error::Backend {
            release_error: Some(_),
            ..
        })
    ));
    assert!(scheduler.cancel().is_err());
    assert_eq!(scheduler.submit(&[(1, down())]), Err(Error::Halted));
}

#[test]
fn unsupported_actions_are_not_silently_accepted() {
    struct Empty;
    impl Backend for Empty {
        type Error = &'static str;
        fn supports(&self, _: Capability) -> bool {
            false
        }
        fn apply(&mut self, _: u64, _: InputFrame) -> Result<(), Self::Error> {
            unreachable!()
        }
        fn release_all(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }
    let mut scheduler = Scheduler::new(Empty, Direct, 1);
    assert_eq!(
        scheduler.submit(&[(0, down())]),
        Err(Error::Unsupported(Capability::Buttons))
    );
}

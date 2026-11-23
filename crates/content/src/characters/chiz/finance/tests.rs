use super::*;
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use crate::simulation::{ExactFraction, state::GameSimulatorState};

struct Fixture {
    request: GameScenarioRequest,
    state: GameSimulatorState,
    queue: BinaryHeap,
    sequence: u64,
    trace: Vec<crate::simulation::GameEventReceipt>,
}

#[test]
fn drawn_principal_stops_interest_and_repayment_restarts_surplus_phase_even_before_positive_deposit()
 {
    let mut fixture = Fixture::new();
    for model in [
        super::super::clock::MODEL,
        super::super::pressed::MODEL,
        super::super::pressed::held::MODEL,
    ] {
        fixture
            .request
            .model_assumptions
            .insert(model.into(), model.description().into());
    }
    super::super::resource::initialize(&fixture.request.actors[0], &mut fixture.state.actors[0])
        .unwrap();
    initialize(
        &fixture.request,
        &fixture.request.actors[0],
        &mut fixture.state.actors[0],
    )
    .unwrap();
    super::super::combat::deposit(&fixture.request.actors[0], &mut fixture.state.actors[0], 0)
        .unwrap();
    fixture.deposit(100, 1);
    fixture.refresh_at(0);
    fixture.state.actors[0]
        .character
        .chiz_mut()
        .unwrap()
        .finance
        .as_mut()
        .unwrap()
        .spend(wide_fraction(ExactFraction::new(200, 1).unwrap()).unwrap())
        .unwrap();
    fixture.refresh_at(500_000);
    assert!(fixture.queue.is_empty());
    fixture.state.actors[0]
        .character
        .chiz_mut()
        .unwrap()
        .finance
        .as_mut()
        .unwrap()
        .acquire(ExactFraction::new(100, 1).unwrap())
        .unwrap();
    fixture.refresh_at(1_200_000);
    assert!(fixture.queue.is_empty());
    fixture.deposit(10, 1);
    fixture.refresh_at(1_300_000);
    let due = fixture.queue.pop().unwrap();
    assert_eq!(due.time_us, 2_200_000);
    fixture.state.current_time_us = due.time_us;
    let GameEventData::Content(crate::callbacks::ContentEvent::Chiz(
        super::super::events::Event::Interest { owner, generation },
    )) = due.data
    else {
        panic!("interest deadline missing");
    };
    interest(&mut fixture.context(), owner, generation).unwrap();
    assert_eq!(
        fixture.state.actors[0]
            .character
            .chiz_mut()
            .unwrap()
            .finance
            .as_ref()
            .unwrap()
            .wallet
            .current(),
        &wide_fraction(ExactFraction::new(11, 1).unwrap()).unwrap()
    );
}

impl Fixture {
    fn new() -> Self {
        let mut request = crate::dsl::compiler::compile_scenario(
            "options duration=5;
            chiz char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            target hp=100000000 def=1080 resist=0; active chiz;",
            Default::default(),
        )
        .unwrap()
        .request;
        request.actors[0].awakening = ntesim_data::AwakeningSelection::selected([2, 4]).unwrap();
        request
            .model_assumptions
            .insert(MODEL.into(), MODEL.description().into());
        let mut state = GameSimulatorState::new(&request);
        super::super::resource::initialize(&request.actors[0], &mut state.actors[0]).unwrap();
        initialize(&request, &request.actors[0], &mut state.actors[0]).unwrap();
        super::super::combat::deposit(&request.actors[0], &mut state.actors[0], 0).unwrap();
        Self {
            request,
            state,
            queue: BinaryHeap::new(),
            sequence: 0,
            trace: Vec::new(),
        }
    }
    fn context(&mut self) -> CallbackContext<'_> {
        CallbackContext {
            state: &mut self.state,
            request: &self.request,
            queue: &mut self.queue,
            sequence: &mut self.sequence,
            trace: &mut self.trace,
        }
    }
    fn refresh_at(&mut self, time: u64) {
        self.state.current_time_us = time;
        refresh(&mut self.context(), 0).unwrap();
    }
    fn deposit(&mut self, n: u128, d: u128) {
        self.state.actors[0]
            .character
            .chiz_mut()
            .unwrap()
            .finance
            .as_mut()
            .unwrap()
            .wallet
            .grant(wide_fraction(ExactFraction::new(n, d).unwrap()).unwrap())
            .unwrap();
    }
    fn tick(&mut self, time: u64) {
        let event = self.queue.pop().unwrap();
        assert_eq!(event.time_us, time);
        self.state.current_time_us = time;
        let GameEventData::Content(crate::callbacks::ContentEvent::Chiz(event)) = event.data else {
            panic!("expected financial callback");
        };
        super::super::events::apply(&mut self.context(), event).unwrap();
    }
    fn balance(&mut self) -> num_rational::BigRational {
        self.state.actors[0]
            .character
            .chiz_mut()
            .unwrap()
            .finance
            .as_ref()
            .unwrap()
            .wallet
            .current()
            .clone()
    }
}

#[test]
fn a2_hundred_becomes_110_then_121_without_half_second_or_a4_multiplication() {
    let mut f = Fixture::new();
    f.deposit(100, 1);
    f.refresh_at(0);
    assert_eq!(f.queue.peek().unwrap().time_us, 1_000_000);
    f.tick(1_000_000);
    assert_eq!(
        f.balance(),
        wide_fraction(ExactFraction::new(110, 1).unwrap()).unwrap()
    );
    f.tick(2_000_000);
    assert_eq!(
        f.balance(),
        wide_fraction(ExactFraction::new(121, 1).unwrap()).unwrap()
    );
    assert_eq!(f.trace.len(), 2);
}

#[test]
fn revival_resets_surplus_phase_even_when_wallet_is_initially_empty_and_stale_generations_cannot_credit()
 {
    let mut f = Fixture::new();
    let hp = f.state.actors[0].hp;
    f.refresh_at(0);
    assert!(f.queue.is_empty());
    f.state.actors[0].hp = 0;
    f.refresh_at(500_000);
    f.state.actors[0].hp = hp;
    f.refresh_at(600_000);
    f.deposit(100, 1);
    f.refresh_at(900_000);
    assert_eq!(f.queue.peek().unwrap().time_us, 1_600_000);
    f.state.current_time_us = 1_600_000;
    interest(&mut f.context(), 0, 0).unwrap();
    assert_eq!(
        f.balance(),
        wide_fraction(ExactFraction::new(100, 1).unwrap()).unwrap()
    );
    f.tick(1_600_000);
    f.tick(2_600_000);
    assert_eq!(
        f.balance(),
        wide_fraction(ExactFraction::new(121, 1).unwrap()).unwrap()
    );
}

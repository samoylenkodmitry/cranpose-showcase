use std::{rc::Rc, task::Poll, time::Duration};

mod support;

use coroflow::{Flow, MainScope, TestScheduler, Turbine};
use cranpose_showcase::{
    data::{
        facts::{catalogue_fact, CatalogueFacts, FactSource},
        favorites::Favorites,
    },
    model::{BODIES, EARTH, MARS, SUN},
    presentation::{body_card_view_model::BodyCardViewModel, screen_messages::ScreenMessages},
};
use support::FakeFacts;

#[test]
fn favorites_toggle_one_body_at_a_time() {
    let scheduler = TestScheduler::new();
    let favorites = Favorites::new(BODIES.len());
    let earth = favorites.is_saved(EARTH);
    let mut earth = scheduler.turbine(&earth);
    assert_eq!(earth.await_item(), false);

    assert!(favorites.toggle(EARTH));
    assert_eq!(earth.await_item(), true);
    assert!(favorites.toggle(MARS));
    earth.expect_no_events();
    assert!(!favorites.toggle(EARTH));
    assert_eq!(earth.await_item(), false);
    assert_eq!(
        favorites
            .saved()
            .value()
            .iter()
            .filter(|saved| **saved)
            .count(),
        1,
        "only Mars is left"
    );
}

#[test]
fn the_catalogue_answers_after_its_latency() {
    let scheduler = TestScheduler::new();
    let facts = CatalogueFacts::new(Duration::from_millis(600));
    assert_eq!(
        scheduler.block_on(facts.fact(EARTH)),
        Ok(catalogue_fact(EARTH))
    );
    assert_eq!(scheduler.now(), Duration::from_millis(600));
    assert!(catalogue_fact(EARTH).contains(BODIES[EARTH].year_length));
    assert_eq!(
        catalogue_fact(SUN),
        format!("A day lasts {}.", BODIES[SUN].day_length),
        "a body with no year leaves it out"
    );
    assert_eq!(
        catalogue_fact(BODIES.len()),
        "",
        "an unknown body has no fact"
    );
}

#[test]
fn a_card_fetches_its_fact_once_and_reports_through_the_screen_bus() {
    let scheduler = TestScheduler::new();
    let favorites = Favorites::new(BODIES.len());
    let facts = FakeFacts::default();
    let messages = Rc::new(ScreenMessages::default());
    let card = BodyCardViewModel::new(
        MainScope::new(scheduler.main_dispatcher()),
        EARTH,
        favorites.clone(),
        Rc::new(facts.clone()),
        Rc::clone(&messages),
    );
    let mut sent = Turbine::of(&messages.events());

    let shown = card.fact().open();
    scheduler.run_current();
    assert_eq!(card.fact().value(), Some(format!("fact {EARTH}")));
    drop(shown);
    scheduler.advance_time_by(Duration::from_secs(60));
    let _back = card.fact().open();
    scheduler.run_current();
    assert_eq!(facts.lookups.get(), 1, "the card keeps the fact it fetched");

    let _saved = card.saved().open();
    card.on_toggle_saved();
    scheduler.run_current();
    assert!(card.saved().value());
    assert_eq!(sent.next_now(), Poll::Ready(Some("Saved Earth".to_owned())));
    card.on_toggle_saved();
    scheduler.run_current();
    assert!(!card.saved().value());
    assert_eq!(
        sent.next_now(),
        Poll::Ready(Some("Removed Earth".to_owned()))
    );
}

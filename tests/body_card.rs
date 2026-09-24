//! Composes the real `BodyCard` against a view model store seeded with a view
//! model built from fakes: how to test a self-contained part of the UI
//! without the rest of the app.

#![allow(dead_code, non_snake_case)]

#[path = "../src/data/mod.rs"]
mod data;
#[path = "../src/model.rs"]
mod model;
#[path = "../src/motion.rs"]
mod motion;
#[path = "../src/presentation/mod.rs"]
mod presentation;
#[path = "../src/screens/mod.rs"]
mod screens;
#[path = "../src/widgets/mod.rs"]
mod widgets;

use std::{cell::Cell, rc::Rc};

use coroflow::MainScope;
use cranpose_coroflow::{main_dispatcher, rememberHandle, ProvideViewModelStore, ViewModelStore};
use cranpose_testing::ComposeTestRule;

use data::{
    facts::{FactFuture, FactSource},
    AppServices,
};
use model::EARTH;
use motion::rememberAmbientMotion;
use presentation::{body_card_view_model::BodyCardViewModel, screen_messages::ScreenMessages};
use screens::list_screen::BodyCard;

struct FakeFacts {
    lookups: Rc<Cell<usize>>,
}

impl FactSource for FakeFacts {
    fn fact(&self, _index: usize) -> FactFuture {
        self.lookups.set(self.lookups.get() + 1);
        Box::pin(std::future::ready("a fake fact".to_owned()))
    }
}

#[test]
fn a_card_shows_the_view_model_its_screen_store_holds() {
    let store = ViewModelStore::default();
    let lookups = Rc::new(Cell::new(0));
    let mut rule = ComposeTestRule::new();
    let (seeded, counter) = (store.clone(), Rc::clone(&lookups));
    rule.set_content(move || {
        let services = rememberHandle(AppServices::default);
        if seeded.get::<usize, BodyCardViewModel>(&EARTH).is_none() {
            let dispatcher = main_dispatcher().expect("the test composition's runtime");
            let messages = seeded.put((), ScreenMessages::default());
            seeded.put(
                EARTH,
                BodyCardViewModel::new(
                    MainScope::new(dispatcher),
                    EARTH,
                    services.get().favorites.clone(),
                    Rc::new(FakeFacts {
                        lookups: Rc::clone(&counter),
                    }),
                    messages,
                ),
            );
        }
        let ambient = rememberAmbientMotion(false);
        ProvideViewModelStore(seeded.clone(), move || {
            BodyCard(EARTH, services, ambient, || {});
        });
    })
    .expect("compose");
    for _ in 0..4 {
        rule.runtime_handle().drain_ui();
        rule.pump_until_idle().expect("pump");
    }

    assert_eq!(
        lookups.get(),
        1,
        "the card collected the fact of the view model its store holds"
    );
    let card = store.get::<usize, BodyCardViewModel>(&EARTH);
    assert_eq!(
        card.and_then(|card| card.fact().value()),
        Some("a fake fact".to_owned())
    );
}

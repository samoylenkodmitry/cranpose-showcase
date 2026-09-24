use std::{rc::Rc, time::Duration};

use coroflow::{flow, FlowExt, MainScope, SharingStarted, StateFlow};

use crate::{
    data::{facts::FactSource, favorites::Favorites},
    model::BODIES,
    presentation::screen_messages::ScreenMessages,
};

/// How long a card's saved flag keeps following the repository after the
/// card stops showing it, so a quick scroll back restarts nothing.
const STOP_TIMEOUT: Duration = Duration::from_secs(5);

/// The view model of one body's card.
///
/// Every card asks for its own, keyed by the body, from the screen's view
/// model store. The list hands the card nothing but the body's index: the
/// card is a self-contained part of the UI whose business logic is scoped to
/// it, and it outlives the card scrolling out of view, so the fact it
/// fetched is still there when the card comes back.
pub struct BodyCardViewModel {
    index: usize,
    favorites: Favorites,
    messages: Rc<ScreenMessages>,
    saved: StateFlow<bool>,
    fact: StateFlow<Option<String>>,
    _scope: MainScope,
}

impl BodyCardViewModel {
    /// The view model of body `index`, whose coroutines run in `scope`.
    pub fn new(
        scope: MainScope,
        index: usize,
        favorites: Favorites,
        facts: Rc<dyn FactSource>,
        messages: Rc<ScreenMessages>,
    ) -> Self {
        let saved = favorites.is_saved(index).state_in(
            &scope,
            SharingStarted::while_subscribed(STOP_TIMEOUT),
            favorites
                .saved()
                .value()
                .get(index)
                .copied()
                .unwrap_or(false),
        );
        let fact = flow(async move |emitter| {
            emitter.emit(Some(facts.fact(index).await)).await;
        })
        .state_in(&scope, SharingStarted::Lazily, None);
        Self {
            index,
            favorites,
            messages,
            saved,
            fact,
            _scope: scope,
        }
    }

    /// Whether the body is saved.
    pub fn saved(&self) -> StateFlow<bool> {
        self.saved.clone()
    }

    /// A fact about the body, fetched once when the card is first shown.
    pub fn fact(&self) -> StateFlow<Option<String>> {
        self.fact.clone()
    }

    /// The user tapped the card's star.
    pub fn on_toggle_saved(&self) {
        let name = BODIES.get(self.index).map_or("", |body| body.name);
        let message = if self.favorites.toggle(self.index) {
            format!("Saved {name}")
        } else {
            format!("Removed {name}")
        };
        self.messages.send(message);
    }
}

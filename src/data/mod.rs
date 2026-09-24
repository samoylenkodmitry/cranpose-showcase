//! Where the app's data comes from. Nothing here knows about Cranpose, so it
//! is tested without a UI.

pub mod facts;
pub mod favorites;

use std::{rc::Rc, time::Duration};

use crate::model::BODIES;

use self::{
    facts::{CatalogueFacts, FactSource},
    favorites::Favorites,
};

/// How long the catalogue takes to answer, as a remote API would.
const FACT_LATENCY: Duration = Duration::from_millis(600);

/// The app's object graph, built once at the root and handed to view model
/// factories.
pub struct AppServices {
    /// The saved bodies.
    pub favorites: Favorites,
    /// Where facts about bodies come from.
    pub facts: Rc<dyn FactSource>,
}

impl Default for AppServices {
    fn default() -> Self {
        Self {
            favorites: Favorites::new(BODIES.len()),
            facts: Rc::new(CatalogueFacts::new(FACT_LATENCY)),
        }
    }
}

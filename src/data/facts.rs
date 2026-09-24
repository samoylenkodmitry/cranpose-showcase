use std::{future::Future, pin::Pin, time::Duration};

use crate::model::BODIES;

/// A fact still on its way.
pub type FactFuture = Pin<Box<dyn Future<Output = String>>>;

/// Looks up a fact about a body. The app's source answers after a network-
/// like delay; tests pass a fake.
pub trait FactSource {
    /// A fact about body `index`.
    fn fact(&self, index: usize) -> FactFuture;
}

/// Facts from the built-in catalogue, answered after `latency`, the way a
/// remote API would.
pub struct CatalogueFacts {
    latency: Duration,
}

impl CatalogueFacts {
    /// A source that answers after `latency`.
    pub fn new(latency: Duration) -> Self {
        Self { latency }
    }
}

impl FactSource for CatalogueFacts {
    fn fact(&self, index: usize) -> FactFuture {
        let latency = self.latency;
        Box::pin(async move {
            coroflow::delay(latency).await;
            catalogue_fact(index)
        })
    }
}

/// The catalogue's fact about body `index`.
pub fn catalogue_fact(index: usize) -> String {
    BODIES
        .get(index)
        .map_or_else(String::new, |body| match body.year_length {
            "" | "—" => format!("A day lasts {}.", body.day_length),
            year => format!("A day lasts {}; a year, {year}.", body.day_length),
        })
}

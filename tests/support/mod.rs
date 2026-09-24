#![allow(dead_code)]

use std::{cell::Cell, rc::Rc};

use cranpose_showcase::data::facts::{FactFuture, FactSource};

/// A fact source that answers at once and counts its lookups.
#[derive(Clone, Default)]
pub struct FakeFacts {
    pub lookups: Rc<Cell<usize>>,
}

impl FactSource for FakeFacts {
    fn fact(&self, index: usize) -> FactFuture {
        self.lookups.set(self.lookups.get() + 1);
        Box::pin(std::future::ready(format!("fact {index}")))
    }
}

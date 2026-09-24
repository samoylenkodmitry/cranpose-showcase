use coroflow::{Flow, FlowExt, MutableStateFlow, StateFlow};

/// Which bodies the user saved. One instance lives as long as the app and
/// every screen reads it: a repository in the Android sense.
#[derive(Clone)]
pub struct Favorites {
    saved: MutableStateFlow<Vec<bool>>,
}

impl Favorites {
    /// Nothing saved yet, out of `count` bodies.
    pub fn new(count: usize) -> Self {
        Self {
            saved: MutableStateFlow::new(vec![false; count]),
        }
    }

    /// One flag per body, `true` where it is saved.
    pub fn saved(&self) -> StateFlow<Vec<bool>> {
        self.saved.as_state_flow()
    }

    /// Whether body `index` is saved, emitted each time that changes.
    pub fn is_saved(&self, index: usize) -> impl Flow<Item = bool> + Clone + 'static {
        self.saved
            .as_state_flow()
            .map(move |saved| saved.get(index).copied().unwrap_or(false))
            .distinct_until_changed()
    }

    /// Saves body `index`, or unsaves it when it is saved, and returns
    /// whether it is saved now.
    pub fn toggle(&self, index: usize) -> bool {
        let saved = self.saved.update_and_get(|saved| {
            let mut next = saved.clone();
            if let Some(flag) = next.get_mut(index) {
                *flag = !*flag;
            }
            next
        });
        saved.get(index).copied().unwrap_or(false)
    }
}

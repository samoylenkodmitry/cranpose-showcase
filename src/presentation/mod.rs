//! View models: screen state as `StateFlow`s and user intents as methods.
//! They depend on the data layer and coroflow, never on Cranpose, so they are
//! tested on virtual time without a UI.

pub mod body_card_view_model;
pub mod screen_messages;

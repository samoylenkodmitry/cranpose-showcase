# Agent notes for the showcase

This app is the template for structuring a Cranpose app. Read
[App architecture](README.md#app-architecture-composable-scoped-view-models)
before adding a screen or a piece of UI with logic of its own.

- Keep business logic in view models under `src/presentation`. They expose
  `StateFlow`s and intent methods and never import Cranpose.
- A screen gets its store from `NavHost`. A part of a screen with its own
  logic resolves its own view model with `viewModel(key, |scope| ..)`, keyed
  so that many of them can share the screen's store, as `BodyCard` does. Pass
  it an id, not state or callbacks from the screen.
- Parts of a screen talk through a view model in the same store, such as
  `ScreenMessages`, not through the screen.
- Use `ViewModelStoreOwner` for a part whose logic should end when it closes.
- Test view models on virtual time (`tests/view_models.rs`). Test UI parts by
  composing them inside `ProvideViewModelStore` with a store seeded with
  view models built from fakes (`tests/body_card.rs`).

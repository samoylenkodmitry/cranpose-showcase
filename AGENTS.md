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

## Simple speech

Never leave prepositions trailing at the ends of clauses (e.g., use "the version by which..." instead of "the version... by"), and keep modifiers close to the words they modify. Use simple words in their original meaning. no poetic, no jargon. No contrastive sentences. No special symbols. No Trailing Participial Phrases. NO for any of these: Overused Buzzwords, Empty Transition Openers, unnecessary adjectives that try to sell an ordinary fact, list things in triples (e.g., "fast, efficient, and reliable" or "streamline, optimize, and scale")., Not only... but also..., wrap-up summaries that don't add actual data. | **No sales pitch or hype:** Adopt a neutral, engineering-first tone. Never use marketing fluff, exclamation points for enthusiasm, or words designed to "sell" a feature (e.g., *effortless, supercharge, magical, lightning-fast*). State facts and mechanics directly.

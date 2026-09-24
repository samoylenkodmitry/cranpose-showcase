# Showcase Cranpose

**Live demo:** <https://samoylenkodmitry.github.io/cranpose-showcase/>

Showcase Cranpose is a liquid-glass 3D star chart built with
[Cranpose](https://github.com/samoylenkodmitry/cranpose). One Rust codebase
ships to desktop, Android, iOS, and the web.

<table>
<tr>
<td align="center"><b>iOS</b></td>
<td align="center"><b>Android</b></td>
</tr>
<tr>
<td><img src="docs/screenshot-ios.png" width="360"></td>
<td><img src="docs/screenshot-android.png" width="360"></td>
</tr>
</table>

## What it demonstrates

- Liquid Glass surfaces with zero material blur, refraction, a pinned
  blurred-gradient crown, and a floating tab bar.
- A procedural WGSL planet shader with lighting, atmosphere, terrain, cloud,
  and ring variants.
- Compose-style state, keyed lazy lists, scroll-linked parallax, transitions,
  and platform safe-area handling.
- `NavHost` navigation and composable-scoped view models: see
  [App architecture](#app-architecture-composable-scoped-view-models).
- One layout that splits into a list column and a detail pane on a desktop
  window and folds back to a single column on a phone, holding a row to a
  readable width at every size.

## App architecture: composable-scoped view models

The showcase is also the template for how a Cranpose app is structured. It
follows the Android architecture guide, with coroutines and flows from
[`coroflow`](https://crates.io/crates/coroflow):

| Layer | Where | What it holds |
|---|---|---|
| Data | `src/data` | `Favorites` (a repository) and `FactSource` (a remote-like API), built once in `AppServices` |
| Presentation | `src/presentation` | View models: state as `StateFlow`s, user intents as methods, no Cranpose imports |
| UI | `src/screens`, `src/app.rs` | Composables that collect state and call intents |

Three pieces make parts of the UI self-contained:

- **A view model store per screen.** `NavHost` gives every back stack entry
  its own `ViewModelStore`. The screen's view models live until the entry is
  popped, not just while the screen is composed.
- **Keyed view models for parts of a screen.** `BodyCard(index, ..)` resolves
  its own `BodyCardViewModel` with `viewModel(index, |scope| ..)`. The list
  only passes the body's index; the card owns its logic (saving, fetching a
  fact). Because the view model lives in the screen's store, a card scrolled
  out of view and back keeps its fact instead of fetching it again.
- **A screen-scoped bus.** `ScreenMessages` is itself a view model in the
  screen's store (`viewModel((), |_| ScreenMessages::default())`). The screen
  and each card get the same instance without knowing about each other, and
  the screen shows what the cards send as a toast.

For a part that should lose its state when it closes, such as a dialog, wrap
it in `ViewModelStoreOwner(|| ..)`: its view models end with the block.

Tests follow the same split. `tests/view_models.rs` runs view models on
virtual time with `TestScheduler`. `tests/body_card.rs` composes the real
`BodyCard` inside `ProvideViewModelStore` with a store seeded with a view
model built from a fake fact source.

## Run locally

```bash
# Desktop
cargo run --features desktop,renderer-wgpu,logging

# Web
./build-web.sh
cd dist && python3 -m http.server 8080

# Android
(cd android && ./gradlew :app:assembleDebug -PshowcaseAbi=arm64-v8a)

# iOS Simulator
./ios/run-sim.sh
```

## Releases

Tag `vX.Y.Z` when the Cargo, Android, and iOS versions match. GitHub Actions
publishes Linux, macOS, Windows, Android, and iOS artifacts and deploys the
web demo to GitHub Pages.

## License

Apache-2.0. See [LICENSE](LICENSE).

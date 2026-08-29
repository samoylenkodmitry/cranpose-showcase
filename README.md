# Cranpose Orbit

An interactive star-chart you can browse, favorite, and poke at — built to
show what [Cranpose](https://github.com/samoylenkodmitry/cranpose), a
Jetpack-Compose-style declarative UI framework for Rust, looks like as a real
app rather than a widget gallery. Orbit runs from one Rust codebase on iOS,
Android, desktop, and the web, using only the published `cranpose` crates
from crates.io — nothing in this repository points at a local Cranpose
checkout.

<table>
<tr>
<td align="center"><b>iOS Simulator</b></td>
<td align="center"><b>Android Emulator</b></td>
</tr>
<tr>
<td><img src="docs/screenshot-ios.png" width="360"></td>
<td><img src="docs/screenshot-android.png" width="360"></td>
</tr>
</table>

## Why a star chart

Cranpose's own `cranpose-liquid` crate is a from-scratch reimplementation of
iOS 26's "Liquid Glass" material — real refraction, spring-driven motion, a
large-title nav bar that collapses under a frosted band, a floating pill tab
bar. That is the most striking thing the framework can show off, so Orbit is
built almost entirely out of it: `LiquidNavBar`, `LiquidTabBar`, `LiquidCard`,
`LiquidChip`, `LiquidSlider`, `GlassIconButton`. A star chart gives that
material something worth sitting on top of — vivid per-planet color, a short
factual hook for every world, and enough real content (fourteen bodies, five
categories, cross-references between them) to need actual navigation and
state instead of a static screen.

Every planet, moon, and star you see is drawn, not loaded: `PlanetSphere`
(`src/widgets/planet.rs`) shades a lit sphere from three radial gradients and
a specular highlight, and the starfield behind every screen
(`src/widgets/starfield.rs`) is a few hundred procedurally placed, twinkling
points. Nothing here is an image asset.

## What it demonstrates

- **Lists and navigation** — a scrolling `Explore` catalog and a `Saved`
  favorites list share one screen implementation (`src/screens/list_screen.rs`),
  filtered live by a search field and category chips; tapping a card pushes a
  detail screen with its own back stack, driven by a small state machine in
  `src/app.rs` rather than a framework-provided router.
- **Spring-driven motion, not linear fades** — the favorite star overshoots
  and settles with `spring(Spring::DampingRatioHighBouncy, ...)`; the detail
  screen enters and exits with a combined fade + slide transition; the
  floating tab bar's own liquid indicator comes from `LiquidTabBar` for free.
- **A large-title nav bar that actually collapses** — `LiquidNavBar` reads the
  content's live scroll offset and snaps to fully expanded or fully collapsed,
  exactly like `UINavigationBar`, sampling the scrolling content through its
  frosted band.
- **Real interaction, not just display** — a "feel the gravity" slider on
  every detail page rescales a sample weight live by that body's surface
  gravity; a staggered entrance animation brings list cards in one after
  another on first appearance.
- **Cross-platform safe areas** — the whole app reads
  `local_safe_area_insets()` once and insets its chrome by it, so the same
  composition clears the iOS notch/home indicator and Android's edge-to-edge
  system bars without per-platform branches.

## Requirements

- Rust stable with the target(s) you're building for.
- iOS: Xcode, and the `aarch64-apple-ios-sim` / `aarch64-apple-ios` targets.
- Android: `cargo-ndk`, the Android SDK/NDK, and a JDK 17 for Gradle.
- Desktop: a working native graphics environment (Vulkan/Metal/DX12, or
  `renderer-wgpu-gles` on machines without one).

## Running on iOS

```bash
rustup target add aarch64-apple-ios-sim aarch64-apple-ios
./ios/run-sim.sh
# or target a specific simulator:
SIMULATOR_DEVICE="iPhone 17 Pro" ./ios/run-sim.sh
```

`ios/build-app.sh` assembles the `.app` bundle directly from the
`cranpose-orbit-ios` binary — Cranpose's iOS backend is a winit `UIView`
backed by `CAMetalLayer`, so there is no Xcode project and no
Objective-C entry point to maintain.

## Running on Android

The native build, ABI selection, and manifest/activity contributions all come
from the `dev.cranpose.android` Gradle plugin, included straight from wherever
Cargo already resolved the `cranpose` crate (the crates.io registry cache, for
this repository). Install the native build bridge once:

```bash
cargo install cargo-ndk
```

Then, from `android/`:

```bash
./gradlew :app:assembleDebug     # x86_64, for the classic Android Studio emulator
./gradlew :app:assembleRelease   # arm64-v8a + x86_64, for Apple Silicon emulators and real devices
./gradlew installDebug           # or installRelease, to also install it
```

Apple Silicon Macs run arm64 system images by default (Android Studio's
Pixel emulators included), so `assembleDebug`'s x86_64-only output will not
install there — use `assembleRelease` or `installRelease` instead.

## Running on desktop

```bash
cargo run --features desktop,renderer-wgpu,logging
```

## Project layout

```text
cranpose-orbit/
├── src/
│   ├── app.rs             # Theme, navigation state, tab bar, transitions
│   ├── model.rs            # The 14-body catalog (facts, colors, cross-links)
│   ├── motion.rs            # The shared ambient animation bundle
│   ├── screens/             # Explore/Saved list screen, detail screen
│   ├── widgets/              # Procedural planet sphere, starfield
│   ├── lib.rs, main.rs, ios_main.rs   # Android/web, desktop, iOS entry points
├── android/                 # Gradle host; the Cranpose plugin configures it
├── ios/                     # build-app.sh / run-sim.sh — no Xcode project
├── Cargo.toml               # Published-crate dependencies only
└── docs/                    # README screenshots
```

To start your own app from this one: copy the repository, rename the Cargo
package and Android `applicationId`/iOS bundle identifier, and replace
`src/model.rs` and the screens with your own content.

## About Cranpose

This app is pinned to `cranpose = "0.1.104"`, the version on crates.io at the
time of writing. Cranpose's iOS backend recently gained measured
UIScrollView-accurate fling physics, rubber-banding, and bounce — the release
carrying that work had not shipped to crates.io yet when this repository was
built, so the scroll feel you see here is the previous physics model, not
that improvement. Bumping the `cranpose`/`cranpose-*` versions once that
release lands is the one upgrade this starter is already waiting on.

Cranpose itself is pre-alpha and under active development:
<https://github.com/samoylenkodmitry/cranpose>.

## License

Apache-2.0, see [LICENSE](LICENSE).

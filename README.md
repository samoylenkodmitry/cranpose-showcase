# Showcase Cranpose

**Try it live: <https://samoylenkodmitry.github.io/cranpose-showcase/>**

An interactive star-chart you can browse, favorite, and poke at — built to
show what [Cranpose](https://github.com/samoylenkodmitry/cranpose), a
Jetpack-Compose-style declarative UI framework for Rust, looks like as a real
app rather than a widget gallery. Showcase Cranpose runs from one Rust codebase on iOS,
Android, desktop, and the web, using only the published `cranpose` crates
from crates.io — nothing in this repository points at a local Cranpose
checkout.

<table>
<tr>
<td align="center"><b>iOS Simulator</b></td>
<td align="center"><b>Android Device</b></td>
</tr>
<tr>
<td><img src="docs/screenshot-ios.png" width="360"></td>
<td><img src="docs/screenshot-android.png" width="360"></td>
</tr>
</table>

## Why a star chart

Cranpose's own `cranpose-liquid` crate is a from-scratch reimplementation of
iOS 26's "Liquid Glass" material — real refraction, spring-driven motion, a
fixed blurred-gradient crown, a floating pill tab
bar. That is the most striking thing the framework can show off, so Showcase Cranpose is
built almost entirely out of it: `GlassSurface`, `LiquidTabBar`, `LiquidChip`,
`LiquidSlider`, `GlassIconButton`. A star chart gives that
material something worth sitting on top of — vivid per-planet color, a short
factual hook for every world, and enough real content (fourteen bodies, five
categories, cross-references between them) to need actual navigation and
state instead of a static screen.

Every planet, moon, and star you see is drawn, not loaded: `PlanetSphere`
(`src/widgets/planet.rs`) is one runtime WGSL shader, applied as a composited
render effect, shared by all fourteen bodies. Each pixel does
a real analytic ray-sphere intersection and Lambert/specular lighting off a
sun-direction uniform, so every world has an actual day/night terminator
instead of a baked highlight — procedural fbm terrain for rocky bodies,
latitude-banded flow noise for the gas giants, an independently rotating
cloud/haze layer, a ring disc that casts its own shadow onto the sphere, and
atmospheric rim glow. One shared shader module means the renderer compiles
this pipeline once for the whole app, not once per body. The starfield behind
every screen (`src/widgets/starfield.rs`) is a few hundred procedurally
placed, twinkling points. Nothing here is an image asset.

## What it demonstrates

- **A real GPU shader as the hero content, not a decoration** — a composited
  `RuntimeShader` lets a composable's entire visual come from a hand-written
  WGSL fragment shader instead of drawn
  vector shapes; `src/widgets/planet.rs` is the reference for parameterizing
  one shader module over many data-driven variants instead of writing one
  module per variant.
- **Lists and navigation** — a scrolling `Explore` catalog and a `Saved`
  favorites list share one screen implementation (`src/screens/list_screen.rs`),
  filtered live by a search field and category chips; tapping a card pushes a
  detail screen with its own back stack, driven by a small state machine in
  `src/app.rs` rather than a framework-provided router.
- **Spring-driven motion, not linear fades** — the favorite star overshoots
  and settles with `spring(Spring::DampingRatioHighBouncy, ...)`; the detail
  screen enters and exits with a combined fade + slide transition; the
  floating tab bar's own liquid indicator comes from `LiquidTabBar` for free.
- **Scroll-driven stellar depth** — the star field shifts more slowly than
  foreground scrolling, so content has visible parallax without moving the
  glass surfaces.
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
`cranpose-showcase-ios` binary — Cranpose's iOS backend is a winit `UIView`
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
./gradlew :app:assembleDebug                    # x86_64 debug, for the classic Android Studio emulator
./gradlew :app:assembleDebug -PshowcaseAbi=arm64-v8a  # arm64 debug, for Apple Silicon emulators and real devices
./gradlew :app:assembleRelease                  # arm64-v8a + x86_64 release
./gradlew installDebug                          # or installRelease, to also install it
```

Debug builds default to `x86_64` for the classic emulator; pass
`-PshowcaseAbi=arm64-v8a` (or `-PshowcaseAbi=arm64-v8a,x86_64`) for an
Apple-Silicon emulator or a real device. Debug is the fast edit/deploy loop —
prefer it over `assembleRelease` while iterating.

## Running on desktop

```bash
cargo run --features desktop,renderer-wgpu,logging
```

## Running on the web

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
./build-web.sh
cd dist && python3 -m http.server 8080
# open http://localhost:8080
```

`build-web.sh` always builds `--release`; there is no dev/fast mode. The page
prefers WebGPU where the browser has it and falls back to WebGL2 everywhere
else — append `?backend=webgpu` or `?backend=gl` to the URL to force one path
for testing. Every push to `main` rebuilds and redeploys the live demo linked
at the top of this file via `.github/workflows/pages.yml`.

## Project layout

```text
cranpose-showcase/
├── src/
│   ├── app.rs             # Theme, navigation state, tab bar, transitions
│   ├── model.rs            # The 14-body catalog (facts, colors, shader params)
│   ├── motion.rs            # The shared ambient animation bundle
│   ├── screens/             # Explore/Saved list screen, detail screen
│   ├── widgets/              # The planet runtime shader, starfield
│   ├── lib.rs, main.rs, ios_main.rs   # Android/web, desktop, iOS entry points
├── robot-runners/           # Headless visual QA via Cranpose's Robot driver
├── android/                 # Gradle host; the Cranpose plugin configures it
├── ios/                     # build-app.sh / run-sim.sh — no Xcode project
├── index.html, build-web.sh # Web entry point and wasm build script
├── .github/workflows/       # Tag releases and Pages deployments
├── Cargo.toml               # Published-crate dependencies only
└── docs/                    # README screenshots
```

To start your own app from this one: copy the repository, rename the Cargo
package and Android `applicationId`/iOS bundle identifier, and replace
`src/model.rs` and the screens with your own content.

## About Cranpose

This app tracks the published Cranpose crates specified in `Cargo.toml`.

Cranpose itself is pre-alpha and under active development:
<https://github.com/samoylenkodmitry/cranpose>.

## License

Apache-2.0, see [LICENSE](LICENSE).

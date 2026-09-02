//! Headless visual QA for the fourteen planet shaders.
//!
//! Renders every `PlanetSphere` from the real widget code side by side
//! through Cranpose's `Robot` driver and writes one PNG. This needs no
//! visible window and no OS screen-recording permission, since the
//! screenshot comes straight from the GPU render target rather than the
//! screen — the fast loop for judging shader changes without eyeballing a
//! live window.
//!
//! Run with:
//! `cargo run --bin robot-planet-gallery --features robot-preview`
//!
//! `SHOWCASE_ROBOT_OUT_DIR` picks the output directory (default `/tmp`),
//! `SHOWCASE_ROBOT_WAIT_MS` picks how long the animation runs before the
//! capture (default 600ms).

#![allow(non_snake_case)]

#[path = "../src/model.rs"]
mod model;
#[path = "../src/motion.rs"]
#[allow(dead_code)]
mod motion;
#[path = "../src/widgets/planet.rs"]
mod planet;

use std::time::Duration;

use cranpose::liquid::prelude::*;
use cranpose::prelude::*;
use cranpose::{AppLauncher, Robot};
use cranpose_ui::text::TextUnit;

use model::{CelestialBody, BODIES};
use motion::AmbientMotion;
use planet::PlanetSphere;

const COLUMNS: usize = 4;
const CELL: f32 = 132.0;
const SPHERE_SIZE: f32 = 112.0;
const WINDOW_WIDTH: u32 = 700;
const WINDOW_HEIGHT: u32 = 960;

#[composable]
fn PlanetTile(body: &'static CelestialBody, ambient: AmbientMotion) {
    Column(
        Modifier::empty().width(CELL),
        ColumnSpec::default()
            .horizontal_alignment(HorizontalAlignment::CenterHorizontally)
            .vertical_arrangement(LinearArrangement::spaced_by(6.0)),
        move || {
            PlanetSphere(
                Modifier::empty().size(Size {
                    width: SPHERE_SIZE,
                    height: SPHERE_SIZE,
                }),
                body,
                ambient,
            );
            Text(
                body.name,
                Modifier::empty(),
                TextStyle {
                    span_style: SpanStyle {
                        color: Some(Color::from_rgba_u8(255, 255, 255, 220)),
                        font_size: TextUnit::Sp(11.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
            Text(
                body.kind.label(),
                Modifier::empty(),
                TextStyle {
                    span_style: SpanStyle {
                        color: Some(Color::from_rgba_u8(255, 255, 255, 130)),
                        font_size: TextUnit::Sp(9.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
        },
    );
}

#[composable]
fn GalleryBody() {
    let ambient = motion::rememberAmbientMotion(false);

    Box(
        Modifier::empty()
            .fill_max_size()
            .background(Color::from_rgb_u8(6, 8, 16)),
        BoxSpec::default(),
        move || {
            Column(
                Modifier::empty().fill_max_size().padding(20.0),
                ColumnSpec::default().vertical_arrangement(LinearArrangement::spaced_by(16.0)),
                move || {
                    for chunk in BODIES.chunks(COLUMNS) {
                        Row(
                            Modifier::empty(),
                            RowSpec::default()
                                .horizontal_arrangement(LinearArrangement::spaced_by(16.0)),
                            move || {
                                for body in chunk {
                                    PlanetTile(body, ambient);
                                }
                            },
                        );
                    }
                },
            );
        },
    );
}

#[composable]
fn Gallery() {
    LiquidTheme(
        LiquidThemeSpec {
            scheme: SchemeMode::Dark,
            accent: Color::from_rgb_u8(142, 124, 255),
            ..LiquidThemeSpec::default()
        },
        GalleryBody,
    );
}

fn main() {
    let _ = env_logger::try_init();
    let out_dir = std::env::var("SHOWCASE_ROBOT_OUT_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let wait_ms: u64 = std::env::var("SHOWCASE_ROBOT_WAIT_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(600);
    let headless = std::env::var("SHOWCASE_ROBOT_HEADLESS").as_deref() != Ok("0");

    AppLauncher::new()
        .with_title("Robot Planet Gallery")
        .with_size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .with_headless(headless)
        .with_test_driver(move |robot: Robot| {
            std::thread::sleep(Duration::from_millis(wait_ms));
            // The ambient transitions loop forever by design, so
            // `wait_for_idle` would never return; pump a bounded number of
            // frames instead.
            robot.pump_frames(6).expect("advance ambient animation");
            let shot = robot.screenshot().expect("screenshot");
            let path = std::path::Path::new(&out_dir).join("planet-gallery.png");
            let image = image::RgbaImage::from_raw(shot.width, shot.height, shot.pixels.clone())
                .expect("valid screenshot buffer");
            image.save(&path).expect("save screenshot");
            println!("wrote {}", path.display());
            robot.exit().expect("exit robot gallery");
        })
        .run(Gallery);
}

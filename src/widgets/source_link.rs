#![allow(non_snake_case)]

use cranpose::liquid::prelude::*;
use cranpose::prelude::*;

/// Where this app's source lives. The button below is the only place the
/// repository is linked: the web build hands the browser a bare canvas, so a
/// link outside the app would have nowhere to live.
pub const SOURCE_URL: &str = "https://github.com/samoylenkodmitry/cranpose-showcase";

/// A glass pill that opens the Showcase repository in the platform browser.
#[composable]
pub fn SourceLink() {
    let uri_handler = local_uri_handler().current();
    GlassButton(
        Modifier::empty(),
        GlassButtonSpec::glass(),
        move || {
            if let Err(error) = uri_handler.open_uri(SOURCE_URL) {
                log::warn!("could not open {SOURCE_URL}: {error}");
            }
        },
        || {
            Row(
                Modifier::empty(),
                RowSpec::default()
                    .vertical_alignment(VerticalAlignment::CenterVertically)
                    .horizontal_arrangement(LinearArrangement::spaced_by(6.0)),
                || {
                    icons::Icon(icons::SHARE, 15.0, liquid_colors().label);
                    Text(
                        "Source",
                        Modifier::empty(),
                        liquid_typography().subheadline.merge(&TextStyle {
                            span_style: SpanStyle {
                                color: Some(liquid_colors().label),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    );
                },
            );
        },
    );
}

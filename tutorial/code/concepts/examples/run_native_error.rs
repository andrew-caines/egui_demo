//! A creator that can fail. A bad setting stops the app before its window is shown, and
//! `run_native` hands the error back to `main`.
//!
//! Run with `cargo run -p concepts --example run_native_error -- 1.5` (starts at 150 %) or
//! `cargo run -p concepts --example run_native_error -- 10` (fails).

use std::ops::RangeInclusive;

use eframe::egui;

/// The zoom factors this app accepts.
const ZOOM_RANGE: RangeInclusive<f32> = 0.5..=3.0;

/// What can go wrong while reading the zoom setting.
#[derive(Debug, PartialEq)]
enum ZoomError {
    NotANumber(String),
    OutOfRange(f32),
}

impl std::fmt::Display for ZoomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotANumber(text) => write!(f, "zoom {text:?} is not a number"),
            Self::OutOfRange(zoom) => write!(
                f,
                "zoom {zoom} is outside {} to {}",
                ZOOM_RANGE.start(),
                ZOOM_RANGE.end()
            ),
        }
    }
}

impl std::error::Error for ZoomError {}

/// The fallible part of set-up, kept out of the creator so a test can call it without a window.
fn parse_zoom(text: &str) -> Result<f32, ZoomError> {
    let zoom: f32 = text
        .trim()
        .parse()
        .map_err(|_| ZoomError::NotANumber(text.to_owned()))?;
    if ZOOM_RANGE.contains(&zoom) {
        Ok(zoom)
    } else {
        Err(ZoomError::OutOfRange(zoom))
    }
}

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("run_native: set-up that can fail")
            .with_inner_size([440.0, 160.0]),
        ..Default::default()
    };
    // The first command-line argument, if there is one, is the zoom factor.
    let zoom_argument = std::env::args().nth(1);
    eframe::run_native(
        "egui-concepts-run-native-error",
        native_options,
        Box::new(move |creation_context| {
            // `?` boxes a `ZoomError` into the creator's error type.
            let zoom = parse_zoom(zoom_argument.as_deref().unwrap_or("1.0"))?;
            creation_context.egui_ctx.set_zoom_factor(zoom);
            Ok(Box::new(ZoomDemo { zoom }))
        }),
    )
}

struct ZoomDemo {
    zoom: f32,
}

impl eframe::App for ZoomDemo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading(format!("Started at zoom {}", self.zoom));
            ui.label("Run it again with a zoom of 10 and no window opens.");
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_number_in_range_is_accepted() {
        assert_eq!(parse_zoom("1.5"), Ok(1.5));
    }

    #[test]
    fn text_that_is_not_a_number_is_an_error() {
        assert_eq!(
            parse_zoom("large"),
            Err(ZoomError::NotANumber("large".to_owned()))
        );
    }

    #[test]
    fn a_number_out_of_range_is_an_error() {
        assert_eq!(parse_zoom("10"), Err(ZoomError::OutOfRange(10.0)));
    }

    #[test]
    fn the_error_says_what_was_wrong() {
        assert_eq!(
            parse_zoom("10").unwrap_err().to_string(),
            "zoom 10 is outside 0.5 to 3"
        );
    }

    #[test]
    fn the_creator_error_reaches_main_as_app_creation() {
        // What eframe does with the creator's `Err`: wraps it, and `run_native` returns it.
        let boxed: Box<dyn std::error::Error + Send + Sync> =
            Box::new(parse_zoom("10").unwrap_err());
        let error = eframe::Error::AppCreation(boxed);

        assert_eq!(
            error.to_string(),
            "app creation error: zoom 10 is outside 0.5 to 3"
        );
    }
}

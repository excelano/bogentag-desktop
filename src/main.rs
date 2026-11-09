// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

#![deny(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod civil;
mod day;
mod demo;
mod dial;
mod display;
mod glyph;
mod palette;
#[cfg(feature = "screenshot")]
mod screenshot;
mod settings;
mod stopwatch;
mod system;
mod time;
mod timer;
mod timing;
mod tour;
mod widgets;

use eframe::egui::{self, Color32, CornerRadius, Stroke, Visuals};

use crate::app::{App, Demo};
use crate::display::{Precision, Scale};
use crate::settings::Place;

const APP_ID: &str = "bogentag";
const PRODUCT: &str = "Bogentag";

fn argument(flag: &str) -> Option<String> {
    let arguments: Vec<String> = std::env::args().collect();
    arguments
        .iter()
        .position(|argument| argument == flag)
        .and_then(|i| arguments.get(i + 1).cloned())
}

fn demo() -> Option<Demo> {
    if !std::env::args().any(|argument| argument == "--demo") {
        return None;
    }
    let place = argument("--place")
        .and_then(|text| {
            let (latitude, longitude) = text.split_once(',')?;
            Some(Place {
                latitude: latitude.trim().parse().ok()?,
                longitude: longitude.trim().parse().ok()?,
            })
        })
        .unwrap_or(demo::PLACE);
    let now = argument("--at")
        .and_then(|text| bz::Instant::parse_rfc3339(&text).ok())
        .or_else(|| bz::Instant::parse_rfc3339(demo::INSTANT).ok())?;
    Some(Demo {
        place,
        now,
        scale: argument("--dial").as_deref().and_then(Scale::parse),
        precision: argument("--precision")
            .as_deref()
            .and_then(Precision::parse),
    })
}

fn visuals() -> Visuals {
    let mut visuals = Visuals::dark();
    visuals.panel_fill = palette::BEZEL;
    visuals.window_fill = palette::ASTRONOMICAL_TWILIGHT;
    visuals.extreme_bg_color = Color32::from_rgb(0x07, 0x0D, 0x19);
    visuals.faint_bg_color = palette::ASTRONOMICAL_TWILIGHT;
    visuals.window_stroke = Stroke::new(1.0, palette::NAUTICAL_TWILIGHT);
    visuals.window_corner_radius = CornerRadius::same(10);
    visuals.override_text_color = Some(palette::LINE);
    visuals.hyperlink_color = palette::SUN;
    visuals.selection.bg_fill = palette::with_opacity(palette::SUN, 0.35);
    visuals.selection.stroke = Stroke::new(1.0, palette::SUN);
    visuals.widgets.inactive.weak_bg_fill = palette::NAUTICAL_TWILIGHT;
    visuals.widgets.inactive.bg_fill = palette::NAUTICAL_TWILIGHT;
    visuals.widgets.hovered.weak_bg_fill = palette::CIVIL_TWILIGHT;
    visuals.widgets.hovered.bg_fill = palette::CIVIL_TWILIGHT;
    visuals.widgets.active.weak_bg_fill = palette::DAY;
    visuals.widgets.active.bg_fill = palette::DAY;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, palette::NAUTICAL_TWILIGHT);
    visuals
}

fn main() -> eframe::Result {
    let demo = demo();
    let viewport = egui::ViewportBuilder::default()
        .with_app_id(APP_ID)
        .with_title(PRODUCT)
        .with_inner_size([980.0, 680.0])
        .with_min_inner_size([560.0, 460.0]);
    #[cfg(feature = "screenshot")]
    let viewport = match screenshot::Screenshot::size() {
        Some(size) => viewport.with_inner_size(size),
        None => viewport,
    };
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        PRODUCT,
        options,
        Box::new(move |cc| {
            cc.egui_ctx.set_visuals(visuals());
            #[allow(unused_mut)]
            let mut app = App::new(demo);
            #[cfg(feature = "screenshot")]
            if let Some(opening) = screenshot::Screenshot::opening() {
                app.open_for_screenshot(&opening);
            }
            Ok(Box::new(app))
        }),
    )
}

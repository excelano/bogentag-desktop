// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

use eframe::egui::{
    self, Align2, Color32, FontId, Painter, Pos2, Rect, RichText, Sense, Stroke, Ui, pos2, vec2,
};

use crate::civil::{self, Clock};
use crate::dial::{self, DialModel, Face};
use crate::display::{Glyph, Position, Precision, Scale};
use crate::glyph::{self, Mark};
use crate::palette::{self, with_opacity};
use crate::widgets;

pub const PAGES: usize = 9;
const VISUAL_HEIGHT: f32 = 300.0;
const QUARTERS: [(&str, f64); 4] = [
    ("Morning", -45.0),
    ("Afternoon", 45.0),
    ("Evening", 135.0),
    ("Night", 225.0),
];
const EIGHTHS: [(&str, f64); 8] = [
    ("Forenoon", -22.5),
    ("Afternoon", 22.5),
    ("Foreset", 67.5),
    ("Afterset", 112.5),
    ("Foremid", 157.5),
    ("Aftermid", 202.5),
    ("Forerise", 247.5),
    ("Afterrise", 292.5),
];

enum Visual {
    Dial(Scale),
    Anchors,
    Offset(Position),
    Notation,
    Wheel(&'static [(&'static str, f64)]),
    DateTurns { date: String, until: String },
    Civil,
}

struct Page {
    title: &'static str,
    text: String,
    visual: Visual,
}

/// The explainer: one page per topic, each with a live piece of the dial
/// computed for the place in use, so the numbers on the page are the
/// reader's own. Returns false once it is closed.
pub fn show(ctx: &egui::Context, page: &mut usize, reading: &bz::Reading, clock: Clock) -> bool {
    let mut open = true;
    let mut finished = false;
    let pages = pages(reading, clock);
    egui::Window::new("About Bogenzeit")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .fixed_size(vec2(540.0, 620.0))
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            let current = &pages[(*page).min(PAGES - 1)];
            let (rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), VISUAL_HEIGHT), Sense::hover());
            visual(ui.painter(), rect, &current.visual, reading, clock);
            ui.add_space(14.0);
            ui.label(
                RichText::new(current.title)
                    .size(22.0)
                    .strong()
                    .color(palette::LINE),
            );
            ui.add_space(8.0);
            glyph_text(ui, &current.text, 15.0, palette::SECONDARY);
            ui.add_space((ui.available_height() - 40.0).max(8.0));
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                if widgets::choice(ui, "Back", false, *page > 0).clicked() {
                    *page -= 1;
                }
                let dots = PAGES as f32 * 14.0;
                ui.add_space(((ui.available_width() - 94.0 - dots) / 2.0).max(0.0));
                let (strip, _) = ui.allocate_exact_size(vec2(dots, 30.0), Sense::hover());
                for i in 0..PAGES {
                    let color = if i == *page {
                        palette::SUN
                    } else {
                        with_opacity(palette::LINE, 0.3)
                    };
                    ui.painter().circle_filled(
                        pos2(strip.left() + 7.0 + 14.0 * i as f32, strip.center().y),
                        3.5,
                        color,
                    );
                }
                ui.add_space((ui.available_width() - 88.0).max(0.0));
                let last = *page + 1 == PAGES;
                if widgets::choice(ui, if last { "Done" } else { "Next" }, true, true).clicked() {
                    if last {
                        finished = true;
                    } else {
                        *page += 1;
                    }
                }
            });
        });
    if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) && *page + 1 < PAGES {
        *page += 1;
    }
    if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) && *page > 0 {
        *page -= 1;
    }
    open && !finished && !ctx.input(|i| i.key_pressed(egui::Key::Escape))
}

fn day_span(day: &bz::SolarDay) -> Option<String> {
    day.day_span_s().map(|span| format!("{} s", span.floor()))
}

fn pages(reading: &bz::Reading, clock: Clock) -> Vec<Page> {
    let day = &reading.day;
    let year = day.date.year;
    let solstice = |month| {
        bz::SolarDay::compute(
            bz::Date {
                year,
                month,
                day: 21,
            },
            reading.lat,
            reading.lon,
        )
    };
    let span_today =
        day_span(day).unwrap_or("undefined, the Sun never crosses the horizon".to_string());
    let span_june = day_span(&solstice(6)).unwrap_or("none".to_string());
    let span_december = day_span(&solstice(12)).unwrap_or("none".to_string());
    let noon = clock.time(day.noon);
    let midnight = clock.time(day.midnight_end);
    let date = civil::bz_day(day.date);
    vec![
        Page {
            title: "Time on the Sun's arc",
            text: "Every day the Sun rises, climbs to its highest point, sets, and passes beneath you to its lowest. Bogenzeit reads time off that arc: where the Sun is now, between those four events. The day arc runs from sunrise to sunset and the night arc from sunset to sunrise. The day arc is the Tagbogen in German; the app's name, Bogentag, turns it around.".to_string(),
            visual: Visual::Dial(Scale::Spans),
        },
        Page {
            title: "Two anchors",
            text: format!("{{noon}} is solar noon, the Sun at its highest. {{midnight}} is solar midnight, the Sun at its lowest. A clock puts noon at 12:00 for a whole zone; the Sun keeps its own time at your longitude. Here, today, solar noon is at {noon} and solar midnight at {midnight}."),
            visual: Visual::Anchors,
        },
        Page {
            title: "Spans",
            text: format!("Each half of an arc is 100 spans: sunrise to {{noon}}, {{noon}} to sunset, and the same on either side of {{midnight}}. A span stretches and shrinks with the season. Here a day span is {span_today} today, {span_june} in June and {span_december} in December. The span dial draws every day the same shape; the season shows in the numbers."),
            visual: Visual::Dial(Scale::Spans),
        },
        Page {
            title: "Kiloseconds",
            text: "The fixed scale, the other way to read the same moment. A kilosecond is 1,000 seconds, a little under seventeen minutes; a day is 86.4 of them. The Kiloseconds dial gives the time from the nearest anchor in kiloseconds, so two readings can be compared and a duration added up. Each dial shows its own scale, and Precision sets how many decimals.".to_string(),
            visual: Visual::Offset(Position::of(reading, Scale::Kiloseconds, Precision::Medium)),
        },
        Page {
            title: "Reading the notation",
            text: "The number sits on the side of the glyph where that moment lies. Left of the glyph, the anchor is still to come: 18 {noon} is eighteen spans before solar noon. Right of it, the anchor has passed: {noon} 3 is three spans after. No plus, no minus. At night the same reading runs against {midnight}. Sunrise and sunset have glyphs of their own, {sunrise} and {sunset}, shown in place of the 100 that starts each arc.".to_string(),
            visual: Visual::Notation,
        },
        Page {
            title: "Four quarters",
            text: "Before and after each anchor are the parts of the day you already name: before {noon} the morning, after it the afternoon; before {midnight} the evening, after it the night. The reading gives you these four quarters on its own, in the glyph and the side the number sits on.".to_string(),
            visual: Visual::Wheel(&QUARTERS),
        },
        Page {
            title: "Eight eighths",
            text: "Halve each quarter at its 50-span mark and you have the eight eighths the dial labels, each named for the nearest event: Fore- before it, After- after it. By day: Afterrise, Forenoon, Afternoon, Foreset. By night: Afterset, Foremid, Aftermid, Forerise.".to_string(),
            visual: Visual::Wheel(&EIGHTHS),
        },
        Page {
            title: "The date turns at solar midnight",
            text: format!("A Bogenzeit day runs from one solar midnight to the next, so the date changes at {{midnight}} rather than at 00:00. Tonight {{midnight}} falls at {midnight}; until then it is still {date} here, whatever the calendar says."),
            visual: Visual::DateTurns {
                date: date.clone(),
                until: format!("until {midnight}"),
            },
        },
        Page {
            title: "Click for the clock",
            text: "The dial shows no hours and no minutes. When you need them, click it: UTC, the reference every reading is computed from, then the local clock with its zone, the date, and today's sunrise and sunset, over the dimmed dial. Click again to return to the Sun.".to_string(),
            visual: Visual::Civil,
        },
    ]
}

fn visual(painter: &Painter, rect: Rect, visual: &Visual, reading: &bz::Reading, clock: Clock) {
    let c = rect.center();
    match visual {
        Visual::Dial(scale) => {
            let model = DialModel::new(*reading, *scale);
            let face = Face {
                precision: Precision::Medium,
                readout: true,
                arc_lengths: true,
                arcs: &[],
            };
            dial::paint(painter, dial::square(rect), &model, &face);
        }
        Visual::Anchors => {
            for (glyph, x) in [(Glyph::Noon, -80.0), (Glyph::Midnight, 80.0)] {
                glyph::paint(
                    painter,
                    glyph,
                    pos2(c.x + x, c.y - 16.0),
                    36.0,
                    7.0,
                    palette::LINE,
                );
                painter.text(
                    pos2(c.x + x, c.y + 44.0),
                    Align2::CENTER_TOP,
                    glyph.name(),
                    FontId::proportional(15.0),
                    palette::LABEL,
                );
            }
        }
        Visual::Offset(position) => {
            let side = if position.before { "before" } else { "after" };
            let meaning = format!("{side} {}", position.glyph.name());
            notation(painter, c, position, &meaning);
        }
        Visual::Notation => {
            let examples = [
                (Some("18"), Glyph::Noon, true, "before solar noon"),
                (Some("3"), Glyph::Noon, false, "after solar noon"),
                (None, Glyph::Sunrise, false, "sunrise"),
                (None, Glyph::Sunset, false, "sunset"),
            ];
            for (i, (number, glyph, before, meaning)) in examples.into_iter().enumerate() {
                let position = Position {
                    glyph,
                    number: number.map(str::to_string),
                    before,
                    unit: None,
                };
                notation(
                    painter,
                    pos2(c.x, c.y - 96.0 + 64.0 * i as f32),
                    &position,
                    meaning,
                );
            }
        }
        Visual::Wheel(names) => wheel(painter, rect, names),
        Visual::DateTurns { date, until } => {
            glyph::paint(
                painter,
                Glyph::Midnight,
                pos2(c.x, c.y - 60.0),
                36.0,
                7.0,
                palette::LINE,
            );
            painter.text(
                pos2(c.x, c.y + 10.0),
                Align2::CENTER_TOP,
                date,
                FontId::proportional(24.0),
                palette::LINE,
            );
            painter.text(
                pos2(c.x, c.y + 46.0),
                Align2::CENTER_TOP,
                until,
                FontId::proportional(15.0),
                palette::LABEL,
            );
        }
        Visual::Civil => {
            let model = DialModel::new(*reading, Scale::Kiloseconds);
            let face = Face {
                precision: Precision::Medium,
                readout: false,
                arc_lengths: false,
                arcs: &[],
            };
            let square = dial::square(rect);
            let mut dimmed = painter.clone();
            dimmed.multiply_opacity(0.35);
            dial::paint(&dimmed, square, &model, &face);
            dial::paint_civil(painter, square, reading, reading.instant, clock);
        }
    }
}

/// One line of notation, the number on its side of the glyph, and what it
/// means to the right.
fn notation(painter: &Painter, center: Pos2, position: &Position, meaning: &str) {
    let mark = Mark {
        size: 40.0,
        glyph_radius: 14.0,
        line_width: 3.0,
        spacing: 10.0,
        color: palette::LINE,
    };
    let width = mark.width(painter, position);
    mark.paint(
        painter,
        position,
        pos2(center.x - 150.0 + width / 2.0, center.y),
        width,
    );
    painter.text(
        pos2(center.x + 150.0, center.y),
        Align2::RIGHT_CENTER,
        meaning,
        FontId::proportional(15.0),
        palette::LABEL,
    );
}

/// The day's parts around a ring, ☉ on top and ○ below, each name at its
/// angle clockwise from ☉; the quarter boundaries solid, the eighth ones
/// dotted.
fn wheel(painter: &Painter, rect: Rect, names: &[(&str, f64)]) {
    let c = rect.center();
    let r = rect.width().min(rect.height()) / 2.0 - 44.0;
    let line = Stroke::new(1.0, with_opacity(palette::LINE, 0.5));
    painter.circle_stroke(c, r, Stroke::new(1.0, with_opacity(palette::LINE, 0.25)));
    painter.line_segment([pos2(c.x - r, c.y), pos2(c.x + r, c.y)], line);
    painter.line_segment([pos2(c.x, c.y - r), pos2(c.x, c.y + r)], line);
    if names.len() == 8 {
        for angle in [45.0f32, 135.0] {
            let (sin, cos) = angle.to_radians().sin_cos();
            let from = pos2(c.x - r * sin, c.y + r * cos);
            let to = pos2(c.x + r * sin, c.y - r * cos);
            painter.extend(egui::Shape::dashed_line(
                &[from, to],
                Stroke::new(1.0, with_opacity(palette::LINE, 0.35)),
                2.0,
                3.0,
            ));
        }
    }
    for (glyph, at) in [
        (Glyph::Noon, pos2(c.x, c.y - r - 16.0)),
        (Glyph::Midnight, pos2(c.x, c.y + r + 16.0)),
        (Glyph::Sunrise, pos2(c.x - r - 16.0, c.y)),
        (Glyph::Sunset, pos2(c.x + r + 16.0, c.y)),
    ] {
        glyph::paint(painter, glyph, at, 9.0, 2.2, palette::LINE);
    }
    for &(name, angle) in names {
        let (sin, cos) = (
            angle.to_radians().sin() as f32,
            angle.to_radians().cos() as f32,
        );
        let color: Color32 = if angle > 90.0 && angle < 270.0 {
            palette::LABEL
        } else {
            palette::LINE
        };
        painter.text(
            pos2(c.x + (r + 34.0) * sin, c.y - (r + 22.0) * cos),
            Align2::CENTER_CENTER,
            name,
            FontId::proportional(13.0),
            color,
        );
    }
}

/// Running text with `{noon}`, `{midnight}`, `{sunrise}` and `{sunset}`
/// drawn as glyphs where they stand.
fn glyph_text(ui: &mut Ui, text: &str, size: f32, color: Color32) {
    let space = ui.fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(" ".to_string(), FontId::proportional(size), color)
            .size()
            .x
    });
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 4.0);
        for word in text.split(' ') {
            let mut rest = word;
            while !rest.is_empty() {
                let glyph = [
                    ("{noon}", Glyph::Noon),
                    ("{midnight}", Glyph::Midnight),
                    ("{sunrise}", Glyph::Sunrise),
                    ("{sunset}", Glyph::Sunset),
                ]
                .into_iter()
                .filter_map(|(token, glyph)| rest.find(token).map(|at| (at, token, glyph)))
                .min_by_key(|(at, ..)| *at);
                match glyph {
                    Some((0, token, glyph)) => {
                        widgets::glyph(ui, glyph, size * 0.36, color);
                        rest = &rest[token.len()..];
                    }
                    Some((at, ..)) => {
                        ui.label(RichText::new(&rest[..at]).size(size).color(color));
                        rest = &rest[at..];
                    }
                    None => {
                        ui.label(RichText::new(rest).size(size).color(color));
                        rest = "";
                    }
                }
            }
            ui.add_space(space);
        }
    });
}

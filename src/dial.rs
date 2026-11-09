// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

use eframe::egui::text::{LayoutJob, TextFormat};
use eframe::egui::{Color32, FontId, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};

use crate::civil::{self, Clock};
use crate::display::{Glyph, Position, Precision, Scale};
use crate::glyph::{self, Mark};
use crate::palette::{self, with_opacity};
use crate::timing::{Arc, ArcKind};

/// The design the mockups are drawn to: a dial 450 units across.
const DESIGN: f32 = 450.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tick {
    pub angle: f64,
    pub major: bool,
}

/// The geometry of one dial for one reading. Angles are degrees clockwise
/// from the top, where ☉ sits; ○ is at 180, sunset on the right, sunrise on
/// the left. A chord angle is where the Sun crosses that height in the
/// evening: None when it never gets that low today, 0 when it is that low
/// even at noon. The ring is ticked in what the dial's angle measures.
pub struct DialModel {
    pub reading: bz::Reading,
    pub scale: Scale,
    pub sun_angle: f64,
    pub sunset_angle: Option<f64>,
    pub civil_angle: Option<f64>,
    pub nautical_angle: Option<f64>,
    pub astronomical_angle: Option<f64>,
    pub eighth_angles: Vec<f64>,
    pub ticks: Vec<Tick>,
    pub eighth_arc: Option<(f64, f64)>,
}

impl DialModel {
    pub fn new(reading: bz::Reading, scale: Scale) -> DialModel {
        let day = reading.day;
        let span_geometry = scale == Scale::Spans && day.polar().is_none();
        let evening = |arc: bz::HalfArc| -> Option<f64> {
            match arc {
                bz::HalfArc::AlwaysAbove => None,
                bz::HalfArc::AlwaysBelow => Some(0.0),
                bz::HalfArc::Seconds(s) => {
                    if span_geometry
                        && let (Some(half_day), Some(half_night)) =
                            (day.half_day_s(), day.half_night_evening_s())
                    {
                        return Some(if s <= half_day {
                            s / half_day * 90.0
                        } else {
                            (90.0 + (s - half_day) / half_night * 90.0).min(180.0)
                        });
                    }
                    Some(s / 86_400.0 * 360.0)
                }
            }
        };
        let sunset_angle = evening(day.horizon);

        let sun_angle = match reading.span() {
            Some(span) if span_geometry => match reading.anchor {
                bz::Anchor::Noon => span * 0.9,
                bz::Anchor::Midnight => 180.0 + span * 0.9,
            },
            _ => (reading.instant - day.noon) / 86_400.0 * 360.0,
        };

        let eighth_angles = match sunset_angle {
            Some(h) if day.polar().is_none() => {
                vec![h / 2.0, -h / 2.0, (180.0 + h) / 2.0, -(180.0 + h) / 2.0]
            }
            _ => Vec::new(),
        };

        let ticks = if span_geometry {
            (0..40)
                .map(|i| Tick {
                    angle: f64::from(i) * 9.0,
                    major: i % 5 == 0,
                })
                .collect()
        } else {
            (0..87)
                .map(|i| Tick {
                    angle: f64::from(i) * 1000.0 / 86_400.0 * 360.0,
                    major: i % 5 == 0,
                })
                .filter(|tick| tick.angle < 360.0)
                .collect()
        };

        let eighth_arc = match (sunset_angle, day.polar(), reading.eighth()) {
            (Some(h), None, Some(eighth)) => {
                let rise = 360.0 - h;
                Some(match eighth {
                    bz::Eighth::Afterrise => (rise, (rise + 360.0) / 2.0),
                    bz::Eighth::Forenoon => ((rise + 360.0) / 2.0, 360.0),
                    bz::Eighth::Afternoon => (0.0, h / 2.0),
                    bz::Eighth::Foreset => (h / 2.0, h),
                    bz::Eighth::Afterset => (h, (180.0 + h) / 2.0),
                    bz::Eighth::Foremid => ((180.0 + h) / 2.0, 180.0),
                    bz::Eighth::Aftermid => (180.0, (180.0 + rise) / 2.0),
                    bz::Eighth::Forerise => ((180.0 + rise) / 2.0, rise),
                })
            }
            _ => None,
        };

        DialModel {
            reading,
            scale,
            sun_angle,
            sunset_angle,
            civil_angle: evening(day.civil),
            nautical_angle: evening(day.nautical),
            astronomical_angle: evening(day.astronomical),
            eighth_angles,
            ticks,
            eighth_arc,
        }
    }

    /// The span dial has no span geometry on a polar day and draws
    /// kiloseconds instead.
    pub fn uses_span_geometry(&self) -> bool {
        self.scale == Scale::Spans && self.reading.polar().is_none()
    }

    /// Degrees along the Sun's track for a stretch of time.
    pub fn sweep(&self, seconds: f64, spans: Option<f64>) -> f64 {
        let degrees = match spans {
            Some(spans) if self.uses_span_geometry() => spans * 0.9,
            _ => seconds / 240.0,
        };
        degrees.clamp(0.0, 359.5)
    }
}

/// What a dial draws besides its sky.
pub struct Face<'a> {
    pub precision: Precision,
    pub readout: bool,
    pub arc_lengths: bool,
    pub arcs: &'a [Arc],
}

/// The scale and centre of a square dial in a rect, with the conversion
/// from the design's polar coordinates.
#[derive(Clone, Copy)]
struct Frame {
    c: Pos2,
    s: f32,
}

impl Frame {
    fn of(rect: Rect) -> Frame {
        Frame {
            c: rect.center(),
            s: rect.width().min(rect.height()) / DESIGN,
        }
    }

    fn point(self, angle: f64, radius: f32) -> Pos2 {
        let a = angle.to_radians() as f32;
        pos2(
            self.c.x + radius * self.s * a.sin(),
            self.c.y - radius * self.s * a.cos(),
        )
    }

    fn arc(self, from: f64, to: f64, radius: f32) -> Vec<Pos2> {
        let steps = ((to - from).abs() / 2.0).ceil().max(2.0) as usize;
        (0..=steps)
            .map(|i| self.point(from + (to - from) * i as f64 / steps as f64, radius))
            .collect()
    }
}

pub fn paint(painter: &Painter, rect: Rect, model: &DialModel, face: &Face) {
    let f = Frame::of(rect);
    let (c, s) = (f.c, f.s);

    painter.circle_filled(c, 215.0 * s, palette::BEZEL);

    if let Some((start, end)) = model.eighth_arc {
        painter.add(Shape::line(
            f.arc(start, end, 205.0),
            Stroke::new(14.0 * s, with_opacity(palette::SUN, 0.3)),
        ));
    }

    for tick in &model.ticks {
        let (inner, outer, width, color) = if tick.major {
            (198.0, 212.0, 2.6 * s, palette::MAJOR_TICK)
        } else {
            (201.0, 209.0, 1.4 * s, palette::MINOR_TICK)
        };
        painter.line_segment(
            [f.point(tick.angle, inner), f.point(tick.angle, outer)],
            Stroke::new(width, color),
        );
    }

    sky(painter, f, model);

    for &angle in &model.eighth_angles {
        rounded(
            painter,
            vec![f.point(angle, 179.0), f.point(angle, 195.0)],
            2.5 * s,
            with_opacity(palette::LINE, 0.8),
        );
    }

    let mark = |glyph: Glyph, at: Pos2| {
        glyph::paint(painter, glyph, at, 9.0 * s, 2.2 * s, palette::LINE);
    };
    mark(Glyph::Noon, f.point(0.0, 173.0));
    mark(Glyph::Midnight, f.point(180.0, 173.0));
    if let Some(h) = model.sunset_angle
        && h > 0.0
        && h < 180.0
    {
        let y = chord_y(f, h);
        let x = ((173.0 * s).powi(2) - (c.y - y).powi(2)).max(0.0).sqrt();
        mark(Glyph::Sunrise, pos2(c.x - x, y - 12.0 * s));
        mark(Glyph::Sunset, pos2(c.x + x, y + 12.0 * s));
    }

    for arc in face.arcs {
        let color = match arc.kind {
            ArcKind::Timer => palette::SUN,
            ArcKind::Stopwatch => with_opacity(palette::LINE, 0.7),
        };
        let from = model.sun_angle + arc.from;
        let to = model.sun_angle + arc.to;
        rounded(painter, f.arc(from, to, 160.0), 5.0 * s, color);
        let far = match arc.kind {
            ArcKind::Timer => to,
            ArcKind::Stopwatch => from,
        };
        painter.circle_filled(f.point(far, 160.0), 6.0 * s, color);
    }

    let sun = f.point(model.sun_angle, 160.0);
    painter.circle_filled(sun, 20.0 * s, with_opacity(palette::SUN, 0.25));
    painter.circle_filled(sun, 11.0 * s, palette::SUN);

    if face.readout {
        let disc = disc_radius(s);
        painter.circle(
            c,
            disc,
            with_opacity(palette::NIGHT, 0.8),
            Stroke::new(s, with_opacity(palette::LINE, 0.12)),
        );
        readout(painter, f, model, face.precision);
        if face.arc_lengths && model.scale == Scale::Kiloseconds {
            arc_lengths(painter, f, &model.reading);
        }
    }
}

fn disc_radius(s: f32) -> f32 {
    (100.0 * s).max(50.0)
}

fn chord_y(f: Frame, angle: f64) -> f32 {
    if angle == 0.0 {
        f.c.y - 195.0 * f.s
    } else {
        f.c.y - 160.0 * f.s * (angle.to_radians().cos() as f32)
    }
}

/// The sky inside the ring: night, then each twilight band and the day
/// laid over it from the top down to where the Sun crosses that height.
fn sky(painter: &Painter, f: Frame, model: &DialModel) {
    let radius = 195.0 * f.s;
    let top = f.c.y - radius;
    let bottom = f.c.y + radius;
    painter.circle_filled(f.c, radius, palette::NIGHT);
    let bands = [
        (palette::ASTRONOMICAL_TWILIGHT, model.astronomical_angle),
        (palette::NAUTICAL_TWILIGHT, model.nautical_angle),
        (palette::CIVIL_TWILIGHT, model.civil_angle),
        (palette::DAY, model.sunset_angle),
    ];
    for (color, angle) in bands {
        let lower = angle.map_or(bottom, |angle| chord_y(f, angle));
        if lower > top {
            painter.add(Shape::convex_polygon(
                segment(f.c, radius, lower),
                color,
                Stroke::NONE,
            ));
        }
    }
    if let Some(h) = model.sunset_angle
        && h > 0.0
    {
        let y = chord_y(f, h);
        let half = (radius.powi(2) - (y - f.c.y).powi(2)).max(0.0).sqrt();
        painter.line_segment(
            [pos2(f.c.x - half, y), pos2(f.c.x + half, y)],
            Stroke::new(1.5 * f.s, with_opacity(palette::LINE, 0.55)),
        );
    }
    painter.circle_stroke(
        f.c,
        160.0 * f.s,
        Stroke::new(f.s, with_opacity(palette::LINE, 0.25)),
    );
}

/// The part of a disc above the height `lower`, as a convex outline.
fn segment(c: Pos2, radius: f32, lower: f32) -> Vec<Pos2> {
    let end = ((c.y - lower) / radius).clamp(-1.0, 1.0).acos();
    let steps = 64;
    let right = (0..=steps).map(|i| {
        let a = end * i as f32 / steps as f32;
        pos2(c.x + radius * a.sin(), c.y - radius * a.cos())
    });
    let left: Vec<Pos2> = (1..=steps)
        .rev()
        .map(|i| {
            let a = end * i as f32 / steps as f32;
            pos2(c.x - radius * a.sin(), c.y - radius * a.cos())
        })
        .collect();
    right.chain(left).collect()
}

fn rounded(painter: &Painter, points: Vec<Pos2>, width: f32, color: Color32) {
    if let (Some(&first), Some(&last)) = (points.first(), points.last()) {
        painter.circle_filled(first, width / 2.0, color);
        painter.circle_filled(last, width / 2.0, color);
    }
    painter.add(Shape::line(points, Stroke::new(width, color)));
}

/// Spaced capitals, centred on `center`, as the dial's small labels are set.
pub fn caption(
    painter: &Painter,
    text: &str,
    center: Pos2,
    size: f32,
    color: Color32,
    max_width: f32,
) -> Rect {
    let layout = |size: f32| {
        let mut job = LayoutJob::default();
        job.append(
            text,
            0.0,
            TextFormat {
                font_id: FontId::proportional(size),
                extra_letter_spacing: size * 0.1,
                color,
                ..Default::default()
            },
        );
        painter.layout_job(job)
    };
    let mut galley = layout(size);
    if galley.size().x > max_width {
        galley = layout(size * (max_width / galley.size().x).max(0.7));
    }
    let rect = Rect::from_center_size(center, galley.size());
    painter.galley(rect.min, galley, color);
    rect
}

/// The centre: the Bogenzeit date, the position in the dial's own scale,
/// and the eighth, with the twilight while there is one.
fn readout(painter: &Painter, f: Frame, model: &DialModel, precision: Precision) {
    let s = f.s;
    let reading = &model.reading;
    let width = 2.0 * disc_radius(s) - 6.0;
    let caption_size = (13.0 * s).max(11.0);
    let mark_height = 70.0 * s;
    let gap = 2.0 * s;
    let line = caption_size * 1.3;
    let top = f.c.y - (2.0 * line + mark_height + 2.0 * gap) / 2.0;

    caption(
        painter,
        &civil::bz_day(reading.day.date),
        pos2(f.c.x, top + line / 2.0),
        caption_size,
        palette::LABEL,
        width,
    );

    let position = Position::of(reading, model.scale, precision);
    let size = if position.unit.is_none() {
        64.0
    } else {
        match precision {
            Precision::Low => 60.0,
            Precision::Medium => 52.0,
            Precision::High => 46.0,
        }
    } * s;
    Mark {
        size,
        glyph_radius: 21.0 * s,
        line_width: 4.5 * s,
        spacing: 8.0 * s,
        color: palette::LINE,
    }
    .paint(
        painter,
        &position,
        pos2(f.c.x, top + line + gap + mark_height / 2.0),
        width,
    );

    caption(
        painter,
        &context(reading),
        pos2(f.c.x, top + line + 2.0 * gap + mark_height + line / 2.0),
        caption_size,
        palette::LABEL,
        width,
    );
}

fn context(reading: &bz::Reading) -> String {
    match (reading.eighth(), reading.night_phase) {
        (Some(eighth), Some(phase)) if phase != bz::NightPhase::Night => {
            format!("{} | {}", eighth.name(), phase.name()).to_uppercase()
        }
        (Some(eighth), _) => eighth.name().to_uppercase(),
        (None, _) if reading.polar() == Some(bz::Polar::AlwaysAbove) => "POLAR DAY".to_string(),
        (None, _) => "POLAR NIGHT".to_string(),
    }
}

/// The day arc's length up in the sky and the night arc's down in the
/// night, on the kilosecond dial: on the span dial every arc is 200 spans.
fn arc_lengths(painter: &Painter, f: Frame, reading: &bz::Reading) {
    let size = (11.0 * f.s).max(11.0);
    let width = 300.0 * f.s;
    if let Some(day) = reading.day.tagbogen_s() {
        caption(
            painter,
            &format!("DAY ARC {} ks", kiloseconds(day)),
            pos2(f.c.x, f.c.y - 122.0 * f.s),
            size,
            with_opacity(palette::LINE, 0.9),
            width,
        );
    }
    if let Some(night) = nachtbogen(reading) {
        caption(
            painter,
            &format!("NIGHT ARC {} ks", kiloseconds(night)),
            pos2(f.c.x, f.c.y + 122.0 * f.s),
            size,
            palette::LABEL,
            width,
        );
    }
}

/// Tonight: sunset to the next sunrise.
pub fn nachtbogen(reading: &bz::Reading) -> Option<f64> {
    let next = bz::SolarDay::compute(reading.day.date.plus_days(1), reading.lat, reading.lon);
    Some(reading.day.half_night_evening_s()? + next.half_night_morning_s()?)
}

/// Kiloseconds to one decimal, floored like every readout.
pub fn kiloseconds(seconds: f64) -> String {
    crate::display::decimal((seconds / 100.0).floor() as i64, 1)
}

/// The tapped view: UTC, the reference every reading is computed from,
/// large; the local clock with its zone under it; the civil date; and
/// today's sunrise and sunset by the local clock.
pub fn paint_civil(
    painter: &Painter,
    rect: Rect,
    reading: &bz::Reading,
    now: bz::Instant,
    clock: Clock,
) {
    let f = Frame::of(rect);
    let s = f.s;
    let radius = (120.0 * s).max(56.0);
    painter.circle(
        f.c,
        radius,
        with_opacity(palette::NIGHT, 0.92),
        Stroke::new(s, with_opacity(palette::LINE, 0.12)),
    );
    let width = 2.0 * radius - 16.0 * s;
    let caption_size = (13.0 * s).max(11.0);
    let secondary_size = (15.0 * s).max(13.0);

    let mut job = LayoutJob::default();
    job.append(
        &civil::utc_clock(now),
        0.0,
        TextFormat {
            font_id: FontId::monospace(56.0 * s),
            color: palette::LINE,
            valign: eframe::egui::Align::BOTTOM,
            ..Default::default()
        },
    );
    job.append(
        "UTC",
        6.0 * s,
        TextFormat {
            font_id: FontId::proportional(secondary_size),
            color: palette::SECONDARY,
            valign: eframe::egui::Align::BOTTOM,
            ..Default::default()
        },
    );
    let clock_galley = painter.layout_job(job);
    let clock_height = clock_galley.size().y;
    let line = caption_size * 1.3;
    let local_line = secondary_size * 1.3;
    let gap = 8.0 * s;
    let total = line + gap + clock_height + gap + local_line + gap + line;
    let mut y = f.c.y - total / 2.0;

    caption(
        painter,
        &civil::civil_date(now),
        pos2(f.c.x, y + line / 2.0),
        caption_size,
        palette::LABEL,
        width,
    );
    y += line + gap;
    let clock_rect =
        Rect::from_center_size(pos2(f.c.x, y + clock_height / 2.0), clock_galley.size());
    painter.galley(clock_rect.min, clock_galley, palette::LINE);
    y += clock_height + gap;
    caption(
        painter,
        &clock.zoned(now),
        pos2(f.c.x, y + local_line / 2.0),
        secondary_size,
        palette::SECONDARY,
        width,
    );
    y += local_line + gap;
    caption(
        painter,
        &clock.horizon(&reading.day),
        pos2(f.c.x, y + line / 2.0),
        caption_size,
        palette::LABEL,
        width,
    );
}

/// The side of the largest square that fits, centred.
pub fn square(rect: Rect) -> Rect {
    let side = rect.width().min(rect.height());
    Rect::from_center_size(rect.center(), vec2(side, side))
}

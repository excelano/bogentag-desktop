// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

use std::time::Duration;

use eframe::egui::{
    self, Align, CursorIcon, Key, KeyboardShortcut, Layout, Modifiers, RichText, Sense, Ui,
};

use crate::civil::Clock;
use crate::dial::{self, DialModel, Face};
use crate::display::{Precision, Scale};
use crate::palette;
use crate::settings::{Place, Settings};
use crate::timer::TimerPanel;
use crate::timing::{self, Timing};
use crate::{day, stopwatch, system, tour, widgets};

const QUIT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Q);
const SETTINGS: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Comma);
const ABOUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::F1);
const PANEL_WIDTH: f32 = 320.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Day,
    Timer,
    Stopwatch,
}

impl Panel {
    const ALL: [Panel; 3] = [Panel::Day, Panel::Timer, Panel::Stopwatch];

    fn title(self) -> &'static str {
        match self {
            Panel::Day => "Day",
            Panel::Timer => "Timer",
            Panel::Stopwatch => "Stopwatch",
        }
    }
}

/// A fixed place and instant, for screenshots and checks; nothing is saved.
#[derive(Debug, Clone, Copy)]
pub struct Demo {
    pub place: Place,
    pub now: bz::Instant,
    pub scale: Option<Scale>,
    pub precision: Option<Precision>,
}

pub struct App {
    settings: Settings,
    timing: Timing,
    clock: Clock,
    demo: Option<Demo>,
    panel: Panel,
    civil: bool,
    settings_open: bool,
    tour: Option<usize>,
    latitude: String,
    longitude: String,
    timer_panel: TimerPanel,
    announced: Option<f64>,
    #[cfg(feature = "screenshot")]
    screenshot: Option<crate::screenshot::Screenshot>,
}

impl App {
    pub fn new(demo: Option<Demo>) -> App {
        let (settings, timing, clock) = match demo {
            Some(demo) => {
                let defaults = Settings::default();
                let settings = Settings {
                    place: Some(demo.place),
                    tour_seen: true,
                    scale: demo.scale.unwrap_or(defaults.scale),
                    precision: demo.precision.unwrap_or(defaults.precision),
                    ..defaults
                };
                (settings, Timing::default(), Clock::TwelveHour)
            }
            None => (Settings::load(), Timing::load(), system::clock()),
        };
        let coordinate = |value: Option<f64>| value.map(|v| format!("{v:.4}")).unwrap_or_default();
        let announced = timing
            .timer
            .filter(|timer| timer.is_done(bz::Instant::now()))
            .and_then(|timer| timer.end);
        let tour = (!settings.tour_seen && settings.place.is_some()).then_some(0);
        App {
            latitude: coordinate(settings.place.map(|place| place.latitude)),
            longitude: coordinate(settings.place.map(|place| place.longitude)),
            settings,
            timing,
            clock,
            demo,
            panel: Panel::Day,
            civil: false,
            settings_open: false,
            tour,
            timer_panel: TimerPanel::default(),
            announced,
            #[cfg(feature = "screenshot")]
            screenshot: crate::screenshot::Screenshot::from_environment(),
        }
    }

    #[cfg(feature = "screenshot")]
    pub fn open_for_screenshot(&mut self, what: &str) {
        match what.split_once(':') {
            Some(("about", page)) => {
                self.tour = page.parse().ok().filter(|&page| page < tour::PAGES)
            }
            Some(("sidebar", "closed")) => self.settings.sidebar = false,
            Some(("timer", amount)) => {
                if let (Ok(amount), Some(place)) = (amount.parse(), self.settings.place) {
                    let unit = timing::Unit::of(self.settings.scale);
                    self.timing.start_timer(amount, unit, self.now(), place);
                    self.panel = Panel::Timer;
                }
            }
            _ if what == "settings" => self.settings_open = true,
            _ if what == "civil" => self.civil = true,
            _ if what == "timer" => self.panel = Panel::Timer,
            _ if what == "stopwatch" => self.panel = Panel::Stopwatch,
            _ => {}
        }
    }

    fn now(&self) -> bz::Instant {
        self.demo.map_or_else(bz::Instant::now, |demo| demo.now)
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input_mut(|i| i.consume_shortcut(&QUIT)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&SETTINGS)) {
            self.settings_open = !self.settings_open;
        }
        if ctx.input_mut(|i| i.consume_shortcut(&ABOUT)) && self.settings.place.is_some() {
            self.tour = Some(0);
        }
        if self.tour.is_none() && ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.settings_open = false;
        }
    }

    /// Says once that the timer has ended, through the desktop, and asks for
    /// the window's attention.
    fn announce(&mut self, ctx: &egui::Context, now: bz::Instant) {
        let Some(timer) = self.timing.timer else {
            return;
        };
        if self.demo.is_some() || !timer.is_done(now) || self.announced == timer.end {
            return;
        }
        self.announced = timer.end;
        system::notify("Timer done", &timing::set_text(timer.amount, timer.unit));
        ctx.send_viewport_cmd(egui::ViewportCommand::RequestUserAttention(
            egui::UserAttentionType::Informational,
        ));
    }

    fn tabs(&mut self, ui: &mut Ui) {
        let gap = 6.0;
        let width = (ui.available_width() - gap * 2.0) / 3.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for panel in Panel::ALL {
                let running = match panel {
                    Panel::Day => false,
                    Panel::Timer => self.timing.timer.is_some(),
                    Panel::Stopwatch => self.timing.stopwatch.as_ref().is_some_and(|s| s.running()),
                };
                let selected = self.panel == panel;
                let label = if running && !selected {
                    format!("{} \u{2022}", panel.title())
                } else {
                    panel.title().to_string()
                };
                if widgets::choice_sized(ui, &label, selected, true, width).clicked() {
                    self.panel = panel;
                }
            }
        });
    }

    fn sidebar_toggle(&mut self, ui: &mut Ui) {
        let (symbol, hint) = if self.settings.sidebar {
            ("\u{203A}", "Hide the sidebar")
        } else {
            ("\u{2039}", "Show the sidebar")
        };
        let corner = ui.max_rect().right_top();
        let rect =
            egui::Rect::from_min_size(corner - egui::vec2(40.0, 0.0), egui::vec2(40.0, 40.0));
        if ui
            .put(
                rect,
                egui::Button::new(RichText::new(symbol).size(30.0)).frame(false),
            )
            .on_hover_text(hint)
            .clicked()
        {
            self.settings.sidebar = !self.settings.sidebar;
        }
    }

    fn links(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("About Bogenzeit").clicked() {
                self.tour = Some(0);
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .button(RichText::new("\u{2699}").size(16.0))
                    .on_hover_text("Settings")
                    .clicked()
                {
                    self.settings_open = !self.settings_open;
                }
            });
        });
    }

    fn main(&mut self, ui: &mut Ui, place: Place) {
        let now = self.now();
        let reading = bz::Reading::at(now, place.latitude, place.longitude);

        let mut sidebar = self.settings.sidebar;
        egui::Panel::right("panel")
            .resizable(false)
            .exact_size(PANEL_WIDTH)
            .frame(egui::Frame::side_top_panel(&ui.ctx().global_style()).inner_margin(16))
            .show_collapsible(ui, &mut sidebar, |ui| {
                egui::Panel::top("tabs")
                    .frame(egui::Frame::NONE.inner_margin(egui::Margin {
                        bottom: 16,
                        ..Default::default()
                    }))
                    .show_separator_line(false)
                    .show(ui, |ui| self.tabs(ui));
                egui::Panel::bottom("links")
                    .frame(egui::Frame::NONE)
                    .show_separator_line(false)
                    .show(ui, |ui| self.links(ui));
                match self.panel {
                    Panel::Day => day::show(ui, &reading, self.clock),
                    Panel::Timer => {
                        self.timer_panel
                            .show(ui, &mut self.timing, &mut self.settings, now, place)
                    }
                    Panel::Stopwatch => {
                        stopwatch::show(ui, &mut self.timing, self.settings.scale, now, place)
                    }
                }
            });
        self.settings.sidebar = sidebar;
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(palette::NIGHT).inner_margin(16))
            .show(ui, |ui| {
                let rect = dial::square(ui.available_rect_before_wrap());
                let response = ui
                    .allocate_rect(rect, Sense::click())
                    .on_hover_cursor(CursorIcon::PointingHand);
                if response.clicked() {
                    self.civil = !self.civil;
                }
                let model = DialModel::new(reading, self.settings.scale);
                let arcs = self.timing.arcs(&model, now);
                let face = Face {
                    precision: self.settings.precision,
                    readout: !self.civil,
                    arc_lengths: true,
                    arcs: &arcs,
                };
                if self.civil {
                    let mut dimmed = ui.painter().clone();
                    dimmed.multiply_opacity(0.35);
                    dial::paint(&dimmed, rect, &model, &face);
                    dial::paint_civil(ui.painter(), rect, &reading, now, self.clock);
                } else {
                    dial::paint(ui.painter(), rect, &model, &face);
                }
                self.sidebar_toggle(ui);
            });
        if let Some(page) = &mut self.tour
            && !tour::show(ui.ctx(), page, &reading, self.clock)
        {
            self.tour = None;
            self.settings.tour_seen = true;
        }
        self.announce(ui.ctx(), now);
        ui.ctx().request_repaint_after(Duration::from_secs(1));
    }

    fn settings_window(&mut self, ctx: &egui::Context) {
        let mut open = self.settings_open;
        egui::Window::new("Settings")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(360.0)
            .show(ctx, |ui| {
                section(ui, "Bogenzeit");
                egui::Grid::new("dial")
                    .num_columns(2)
                    .spacing([16.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Dial");
                        egui::ComboBox::from_id_salt("scale")
                            .selected_text(self.settings.scale.title())
                            .show_ui(ui, |ui| {
                                for scale in Scale::ALL {
                                    ui.selectable_value(&mut self.settings.scale, scale, scale.title());
                                }
                            });
                        ui.end_row();
                        ui.label("Precision");
                        egui::ComboBox::from_id_salt("precision")
                            .selected_text(self.settings.precision.title())
                            .show_ui(ui, |ui| {
                                for precision in Precision::ALL {
                                    ui.selectable_value(
                                        &mut self.settings.precision,
                                        precision,
                                        precision.title(),
                                    );
                                }
                            });
                        ui.end_row();
                    });
                note(
                    ui,
                    "Spans draw every day the same shape; Kiloseconds move the Sun evenly and the horizon with the season. Precision is whole spans or up to two decimals, or one to three decimals of a kilosecond.",
                );
                section(ui, "Place");
                if self.demo.is_none() {
                    self.coordinates(ui);
                } else if let Some(place) = self.settings.place {
                    ui.label(format!("{:.2}, {:.2}", place.latitude, place.longitude));
                }
                ui.add_space(12.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION")))
                            .size(11.0)
                            .monospace()
                            .color(palette::LABEL),
                    );
                });
            });
        self.settings_open = open;
    }

    fn coordinates(&mut self, ui: &mut Ui) {
        egui::Grid::new("coordinates")
            .num_columns(2)
            .spacing([16.0, 8.0])
            .show(ui, |ui| {
                ui.label("Latitude");
                ui.add(egui::TextEdit::singleline(&mut self.latitude).desired_width(120.0));
                ui.end_row();
                ui.label("Longitude");
                ui.add(egui::TextEdit::singleline(&mut self.longitude).desired_width(120.0));
                ui.end_row();
            });
        let parsed = (
            self.latitude.trim().parse::<f64>(),
            self.longitude.trim().parse::<f64>(),
        );
        let valid = matches!(parsed, (Ok(lat), Ok(lon)) if (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lon));
        if ui
            .add_enabled(valid, egui::Button::new("Use These"))
            .clicked()
            && let (Ok(latitude), Ok(longitude)) = parsed
        {
            if self.settings.place.is_none() && !self.settings.tour_seen {
                self.tour = Some(0);
            }
            self.settings.place = Some(Place {
                latitude,
                longitude,
            });
        }
        note(
            ui,
            "Degrees, north and east positive. Kept on this computer.",
        );
    }

    fn gate(&mut self, ui: &mut Ui) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(palette::NIGHT))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space((ui.available_height() / 2.0 - 150.0).max(24.0));
                    ui.label(RichText::new("Where you are").size(22.0).strong().color(palette::LINE));
                    ui.add_space(12.0);
                    ui.set_max_width(400.0);
                    ui.label(
                        RichText::new(
                            "Bogentag reads the time from the Sun's arc, and solar noon, sunrise and sunset depend on where you are.",
                        )
                        .color(palette::LABEL),
                    );
                    ui.add_space(16.0);
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width().min(400.0), 0.0),
                        Layout::top_down(Align::Min),
                        |ui| self.coordinates(ui),
                    );
                });
            });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let settings = self.settings.clone();
        let timing = self.timing.clone();
        self.shortcuts(&ctx);
        match self.settings.place {
            Some(place) => self.main(ui, place),
            None => self.gate(ui),
        }
        self.settings_window(&ctx);
        if self.demo.is_none() {
            if self.settings != settings {
                self.settings.save();
            }
            if self.timing != timing {
                self.timing.save();
            }
        }
        #[cfg(feature = "screenshot")]
        if let Some(screenshot) = &mut self.screenshot {
            screenshot.tick(&ctx);
        }
    }
}

fn section(ui: &mut Ui, title: &str) {
    ui.add_space(10.0);
    ui.label(RichText::new(title).size(12.0).strong().color(palette::SUN));
    ui.add_space(2.0);
}

fn note(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).size(11.0).color(palette::LABEL));
}

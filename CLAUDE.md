# CLAUDE.md

Bogentag for the Linux desktop: the Bogentag dial in an egui window. It is the
desktop sibling of excelano/bogentag, the Apple Watch and iPhone app, and
follows that repository's `docs/bogentag-spec.md` for what the dial shows; the
Bogenzeit spec in excelano/bogenzeit is normative. The engine is the `bz` crate
from crates.io.

## Layout

One crate. `dial.rs` is the dial's geometry (`DialModel`, angles clockwise from
☉ at the top, in the 450-unit design of the mockups) and its painting, with the
civil view a click puts over it. `display.rs` holds the display forms, `glyph.rs`
draws the four glyphs and sets numerals, `timing.rs` is the timer and stopwatch
model with `timer.rs` and `stopwatch.rs` its panels, `day.rs` the day panel and
`tour.rs` the About Bogenzeit pages. `system.rs` is the only file that talks to
the desktop; `settings.rs` holds the config directory for each platform.

## Commands

    cargo test
    cargo clippy --all-targets --features screenshot -- -D warnings
    cargo run -- --demo
    cargo run -- --demo --place 78.22,15.65 --at 2026-06-21T10:00:00Z --dial kiloseconds
    cargo check --target x86_64-pc-windows-msvc
    cargo check --target aarch64-apple-darwin
    ./packaging/debian/build-deb.sh
    ./packaging/linux/check-libraries.sh

The `screenshot` feature lets a build photograph its own window:
`BOGENTAG_SCREENSHOT=<png>` saves one after `BOGENTAG_SCREENSHOT_DELAY` seconds
and closes, `BOGENTAG_SCREENSHOT_SIZE=<w>x<h>` sizes the window, and
`BOGENTAG_SCREENSHOT_OPEN` opens `settings`, `civil`, `timer`, `timer:<amount>`
(started), `stopwatch`, `sidebar:closed` or `about:<page>` first. Point
`XDG_CONFIG_HOME` at a scratch directory to run without touching the real
profile.

## Rules

Displayed spans and kiloseconds are the floor of the signed value; position
replaces sign, the number left of the glyph before and right of it after. ☉, ○,
△ and ▽ are drawn by `glyph.rs`, never set in a font, so no glyph characters in
UI strings. No hours or minutes in a Bogenzeit readout; the civil view is the
only clock. The window keeps the night palette whatever the desktop theme. It
makes no network connection.

Every platform-specific call sits behind `cfg(target_os)` in `system.rs` or
`settings.rs`, and the Windows and macOS checks above must stay clean, so the
code is ready for the Microsoft Store and Mac App Store lanes when they are
added. Don't commit, push or publish unless David asks.

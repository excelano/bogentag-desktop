# Bogentag for the Linux desktop

Bogentag shows the time as Bogenzeit, read from where the Sun is on its arc
rather than from hours and minutes. Each half of the arc holds 100 spans:
sunrise to solar noon, solar noon to sunset, and the same on either side of
solar midnight. A reading such as 18 before the noon glyph means eighteen spans
before solar noon where you are, and because a span stretches with the season,
the same reading falls at a different clock time in June and in December but at
the same place in the sky.

This is the desktop sibling of the Bogentag app for Apple Watch and iPhone, and
shows the same dial.

## What it does

The window is the dial: a sky with the day above the horizon, the three
twilight bands below it and the night at the bottom, the Sun on its track, and
the reading in the center with the eighth of the day it falls in. The Spans dial
draws every day the same shape, with sunrise always at nine o'clock and sunset
at three, and the season shows in the twilight bands. The Kiloseconds dial moves
the Sun evenly through a day of 86.4 kiloseconds, so the horizon rises and falls
with the season, and reads the time from the nearest anchor in kiloseconds. Each
dial reads in its own unit only, at the precision chosen in Settings.

A click on the dial shows the clock over it: UTC, the local time with its zone,
the date, and the day's sunrise and sunset. The Day panel gives the lengths of
the day and night arcs and the times of sunrise, solar noon, sunset and solar
midnight.

The timer counts down in spans or kiloseconds. A span timer ends when the Sun
has moved that many spans on, so twenty spans started just before sunset take
ten day spans and ten longer night spans; while it runs, a gold arc on the dial
runs from the Sun to where it ends, and the desktop announces the end. The
stopwatch counts what passes in the dial's unit, with laps. Both are kept across
a restart, though the end of a timer is announced only while the window is
open.

About Bogenzeit, opened on first run and again from the toolbar or with F1,
explains the standard with the numbers for your own place.

## Install

On Debian and Ubuntu, from the Excelano apt repository:

```
curl -fsSL https://excelano.com/apt/setup.sh | sudo sh && sudo apt install bogentag
```

## Building

It is a Rust crate and builds with a current stable toolchain:

    cargo build --release
    ./packaging/debian/build-deb.sh

The second line writes a `.deb` to `dist/` with the desktop entry, icon and
manual page. `cargo run --release -- --demo` opens the Bogenzeit
specification's worked example, Houston on the afternoon of 1 November 2026,
without asking where you are.

## First run

Solar noon, sunrise and sunset depend on where you are, so Bogentag asks for
your latitude and longitude on first run; Settings changes them later.

## Privacy

Bogentag makes no network connection: every reading is computed on this
computer. [What Bogentag stores](SECURITY.md#what-bogentag-stores) lists the
files it keeps.

## The standard

Bogenzeit is defined in [excelano/bogenzeit](https://github.com/excelano/bogenzeit),
which also holds the `bz` engine this app computes with.

## License

MIT. See [LICENSE](LICENSE).

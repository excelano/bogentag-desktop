# Security Policy

## Reporting a vulnerability

Please report suspected vulnerabilities privately through GitHub Security
Advisories at https://github.com/excelano/bogentag-desktop/security/advisories/new.
If you would rather not use GitHub, email david.anderson@excelano.com instead. I
aim to respond within seven days.

Please do not open public issues for security problems.

## Supported versions

The latest 0.x release receives security fixes. Older versions are not
supported.

## What this application is

Bogentag draws the time as the Sun's position for a place the user types in.
It makes no network connection: no telemetry, no update check, no account. Its
only outside contact is the desktop session bus, to read whether the clock is
set to 12 or 24 hours and to post a notification when a timer ends.

## What it stores

The settings, the place and the timer and stopwatch, in
`~/.config/bogentag/settings.json` and `~/.config/bogentag/timing.json`, each
created readable by its owner only. A malformed file there is read as absent,
and is a crash worth reporting if it is not.

## Verifying releases

Every release lists a SHA-256 for each artefact. Verify a download before
running it.

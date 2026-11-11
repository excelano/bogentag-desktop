# Security Policy

## Reporting a vulnerability

Please report suspected vulnerabilities privately, using the **Report a vulnerability** button on the Security tab of this repository. If that button is not there, or you would rather not go through GitHub, email david.anderson@excelano.com instead. I aim to respond within seven days.

Please do not open public issues for security problems.

## Supported versions

The latest release receives security fixes. Older versions are not supported. A fix ships in a new release; update the way you installed.

## What Bogentag can access

Bogentag draws the time as the Sun's position for a place the user types in.
It makes no network connection: no telemetry, no update check, no account. Its
only outside contact is the desktop session bus, to read whether the clock is
set to 12 or 24 hours and to post a notification when a timer ends.

## What Bogentag stores

The settings, the place and the timer and stopwatch, in
`~/.config/bogentag/settings.json` and `~/.config/bogentag/timing.json`, each
created readable by its owner only. A malformed file there is read as absent,
and is a crash worth reporting if it is not.

## Verifying releases

Every release lists a SHA-256 for each artifact. Verify a download before
running it.

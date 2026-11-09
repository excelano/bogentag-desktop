// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

use crate::civil::Clock;

/// The desktop's clock setting, 12- or 24-hour.
pub fn clock() -> Clock {
    #[cfg(target_os = "linux")]
    if linux::clock_format().as_deref() == Some("12h") {
        return Clock::TwelveHour;
    }
    Clock::TwentyFourHour
}

/// A notification from the desktop, sent without waiting for it.
pub fn notify(summary: &str, body: &str) {
    #[cfg(target_os = "linux")]
    linux::notify(summary.to_string(), body.to_string());
    #[cfg(not(target_os = "linux"))]
    let _ = (summary, body);
}

#[cfg(target_os = "linux")]
mod linux {
    use std::collections::HashMap;
    use std::sync::mpsc;
    use std::time::Duration;

    use zbus::blocking::{Connection, Proxy};
    use zbus::zvariant::{OwnedValue, Value};

    use crate::APP_ID;

    /// Asked of the settings portal on a thread, so a session without one
    /// costs a second at startup rather than a hang.
    pub fn clock_format() -> Option<String> {
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = sender.send(read_clock_format());
        });
        receiver.recv_timeout(Duration::from_secs(1)).ok().flatten()
    }

    fn read_clock_format() -> Option<String> {
        let connection = Connection::session().ok()?;
        let settings = Proxy::new(
            &connection,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Settings",
        )
        .ok()?;
        let value: OwnedValue = settings
            .call("ReadOne", &("org.gnome.desktop.interface", "clock-format"))
            .ok()?;
        String::try_from(value).ok()
    }

    pub fn notify(summary: String, body: String) {
        std::thread::spawn(move || -> zbus::Result<()> {
            let connection = Connection::session()?;
            let notifications = Proxy::new(
                &connection,
                "org.freedesktop.Notifications",
                "/org/freedesktop/Notifications",
                "org.freedesktop.Notifications",
            )?;
            let hints: HashMap<&str, Value> = HashMap::from([
                ("desktop-entry", Value::from(APP_ID)),
                ("urgency", Value::U8(1)),
            ]);
            let _: u32 = notifications.call(
                "Notify",
                &(
                    crate::PRODUCT,
                    0u32,
                    APP_ID,
                    summary.as_str(),
                    body.as_str(),
                    Vec::<&str>::new(),
                    hints,
                    -1i32,
                ),
            )?;
            Ok(())
        });
    }
}

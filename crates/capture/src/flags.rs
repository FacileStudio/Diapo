//! The flags `chrome-headless-shell` runs with: quiet, offline, one scale.

use crate::tools::Tools;

const BASE: &[&str] = &[
    "--remote-debugging-pipe",
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-background-networking",
    "--disable-component-update",
    "--disable-default-apps",
    "--disable-extensions",
    "--disable-sync",
    "--disable-breakpad",
    "--disable-dev-shm-usage",
    "--metrics-recording-only",
    "--mute-audio",
    "--hide-scrollbars",
    "--disable-gpu",
    "--force-device-scale-factor=1",
    "--force-color-profile=srgb",
    "--proxy-server=http://127.0.0.1:1",
    "--proxy-bypass-list=<-loopback>",
    "--host-resolver-rules=MAP * ~NOTFOUND",
    "--force-webrtc-ip-handling-policy=disable_non_proxied_udp",
];

/// Every flag for a window of `width` by `height`, without the profile directory. The proxy and
/// resolver flags cut the network under the request guard: every connection goes to a port
/// nothing listens on, and no name resolves.
pub(crate) fn flags(tools: &Tools, (width, height): (u32, u32)) -> Vec<String> {
    let mut flags: Vec<String> = BASE.iter().map(|flag| (*flag).to_owned()).collect();
    flags.push(format!("--window-size={width},{height}"));
    if !tools.sandbox {
        flags.push("--no-sandbox".to_owned());
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_network_is_cut_and_the_sandbox_stays_unless_asked() {
        let tools = Tools { chrome: "chrome".into(), sandbox: true };
        let kept = flags(&tools, (1920, 1080));
        assert!(kept.contains(&"--proxy-server=http://127.0.0.1:1".to_owned()));
        assert!(kept.contains(&"--window-size=1920,1080".to_owned()));
        assert!(!kept.iter().any(|flag| flag == "--no-sandbox"));
        assert!(flags(&Tools { sandbox: false, ..tools }, (2, 2)).contains(&"--no-sandbox".to_owned()));
    }
}

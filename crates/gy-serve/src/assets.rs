//! The static files, embedded (ac-932f); `index.html` carries `%LANG%`.
const INDEX: &str = include_str!("assets/index.html");
const CSS: &str = include_str!("assets/app.css");
const STATE_JS: &str = include_str!("assets/state.js");
const SHAPE_JS: &str = include_str!("assets/shape.js");
const LANES_JS: &str = include_str!("assets/lanes.js");
const READS_JS: &str = include_str!("assets/reads.js");
const WATCH_JS: &str = include_str!("assets/watch.js");
const EVENTS_JS: &str = include_str!("assets/events.js");
const ROOT_JS: &str = include_str!("assets/root.js");
const SIDEBAR_JS: &str = include_str!("assets/sidebar.js");
const TOPBAR_JS: &str = include_str!("assets/topbar.js");
const BAND_JS: &str = include_str!("assets/band.js");
const OVERVIEW_CSS: &str = include_str!("assets/overview.css");
const OVERVIEW_JS: &str = include_str!("assets/overview.js");
const CROSS_CSS: &str = include_str!("assets/cross.css");
const CROSS_JS: &str = include_str!("assets/cross.js");
const NOW_CSS: &str = include_str!("assets/now.css");
const NOW_JS: &str = include_str!("assets/now.js");
const LIST_CSS: &str = include_str!("assets/list.css");
const LIST_JS: &str = include_str!("assets/list.js");
const EYE_CSS: &str = include_str!("assets/eye.css");
const EYE_JS: &str = include_str!("assets/eye.js");
const HISTORY_CSS: &str = include_str!("assets/history.css");
const HISTORY_JS: &str = include_str!("assets/history.js");
const NODE_CSS: &str = include_str!("assets/node.css");
const NODE_JS: &str = include_str!("assets/node.js");
const TIME_CSS: &str = include_str!("assets/time.css");
const TIME_JS: &str = include_str!("assets/time.js");
const WRITES_JS: &str = include_str!("assets/writes.js");
const COPY_JS: &str = include_str!("assets/copy.js");
const RAIL_CSS: &str = include_str!("assets/rail.css");
const RAIL_JS: &str = include_str!("assets/rail.js");
const PALETTE_CSS: &str = include_str!("assets/palette.css");
const PALETTE_JS: &str = include_str!("assets/palette.js");
const I18N: &str = include_str!("assets/i18n.json");

/// Every embedded file: its name and its text.
pub const FILES: [(&str, &str); 35] = [
    ("index.html", INDEX),
    ("app.css", CSS),
    ("state.js", STATE_JS),
    ("shape.js", SHAPE_JS),
    ("lanes.js", LANES_JS),
    ("reads.js", READS_JS),
    ("watch.js", WATCH_JS),
    ("events.js", EVENTS_JS),
    ("root.js", ROOT_JS),
    ("sidebar.js", SIDEBAR_JS),
    ("topbar.js", TOPBAR_JS),
    ("band.js", BAND_JS),
    ("overview.css", OVERVIEW_CSS),
    ("overview.js", OVERVIEW_JS),
    ("cross.css", CROSS_CSS),
    ("cross.js", CROSS_JS),
    ("now.css", NOW_CSS),
    ("now.js", NOW_JS),
    ("list.css", LIST_CSS),
    ("list.js", LIST_JS),
    ("eye.css", EYE_CSS),
    ("eye.js", EYE_JS),
    ("history.css", HISTORY_CSS),
    ("history.js", HISTORY_JS),
    ("node.css", NODE_CSS),
    ("node.js", NODE_JS),
    ("time.css", TIME_CSS),
    ("time.js", TIME_JS),
    ("writes.js", WRITES_JS),
    ("copy.js", COPY_JS),
    ("rail.css", RAIL_CSS),
    ("rail.js", RAIL_JS),
    ("palette.css", PALETTE_CSS),
    ("palette.js", PALETTE_JS),
    ("i18n.json", I18N),
];

/// The one HTML document, with `%LANG%` still in place.
pub fn index() -> &'static str {
    INDEX
}

/// A file under `assets/`: its text and content type. A name with `..` or a
/// separator is not a file (ac-f5c4).
pub fn get(name: &str) -> Option<(&'static str, &'static str)> {
    if name.contains("..") || name.contains('/') {
        return None;
    }
    FILES
        .iter()
        .find(|(file, _)| *file == name)
        .map(|(_, body)| (*body, content_type(name)))
}

/// The content type by extension; the dictionary and the API are both JSON.
fn content_type(name: &str) -> &'static str {
    match name.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        _ => "application/octet-stream",
    }
}

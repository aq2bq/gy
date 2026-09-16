//! The static files, embedded (ac-932f); `index.html` carries `%LANG%`.
const INDEX: &str = include_str!("assets/index.html");
const CSS: &str = include_str!("assets/app.css");
const STATE_JS: &str = include_str!("assets/state.js");
const READS_JS: &str = include_str!("assets/reads.js");
const EVENTS_JS: &str = include_str!("assets/events.js");
const ROOT_JS: &str = include_str!("assets/root.js");
const SIDEBAR_JS: &str = include_str!("assets/sidebar.js");
const TOPBAR_JS: &str = include_str!("assets/topbar.js");
const BAND_JS: &str = include_str!("assets/band.js");
const I18N: &str = include_str!("assets/i18n.json");
const NOW_CSS: &str = include_str!("assets/now.css");
const SKY_JS: &str = include_str!("assets/sky.js");
const NOW_JS: &str = include_str!("assets/now.js");
const PALETTE_CSS: &str = include_str!("assets/palette.css");
const PALETTE_JS: &str = include_str!("assets/palette.js");
const LIST_CSS: &str = include_str!("assets/list.css");
const LIST_JS: &str = include_str!("assets/list.js");
const EYE_CSS: &str = include_str!("assets/eye.css");
const EYE_JS: &str = include_str!("assets/eye.js");
const HISTORY_CSS: &str = include_str!("assets/history.css");
const HISTORY_JS: &str = include_str!("assets/history.js");
const NODE_CSS: &str = include_str!("assets/node.css");
const NODE_JS: &str = include_str!("assets/node.js");
const TIME_CSS: &str = include_str!("assets/time.css");
const GRAPH_CSS: &str = include_str!("assets/graph.css");
const GRAPH_DRAW_JS: &str = include_str!("assets/graph-draw.js");
const GRAPH_JS: &str = include_str!("assets/graph.js");
const WRITES_JS: &str = include_str!("assets/writes.js");
const RAIL_CSS: &str = include_str!("assets/rail.css");
const RAIL_JS: &str = include_str!("assets/rail.js");

/// Every embedded file: its name and its text.
pub const FILES: [(&str, &str); 30] = [
    ("index.html", INDEX),
    ("app.css", CSS),
    ("state.js", STATE_JS),
    ("reads.js", READS_JS),
    ("events.js", EVENTS_JS),
    ("root.js", ROOT_JS),
    ("sidebar.js", SIDEBAR_JS),
    ("topbar.js", TOPBAR_JS),
    ("band.js", BAND_JS),
    ("i18n.json", I18N),
    ("now.css", NOW_CSS),
    ("sky.js", SKY_JS),
    ("now.js", NOW_JS),
    ("palette.css", PALETTE_CSS),
    ("palette.js", PALETTE_JS),
    ("list.css", LIST_CSS),
    ("list.js", LIST_JS),
    ("eye.css", EYE_CSS),
    ("eye.js", EYE_JS),
    ("history.css", HISTORY_CSS),
    ("history.js", HISTORY_JS),
    ("node.css", NODE_CSS),
    ("node.js", NODE_JS),
    ("time.css", TIME_CSS),
    ("graph.css", GRAPH_CSS),
    ("graph-draw.js", GRAPH_DRAW_JS),
    ("graph.js", GRAPH_JS),
    ("writes.js", WRITES_JS),
    ("rail.css", RAIL_CSS),
    ("rail.js", RAIL_JS),
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

//! Where the page finds its websocket.

/// The websocket address for the page at `pathname` on `host`.
///
/// The socket sits beside the page: `/ws` when the page is at the root, and
/// `/editor/ws` when the editor is mounted under `/editor/`. `protocol` is the
/// page's own, `http:` or `https:`.
pub fn websocket_url(protocol: &str, host: &str, pathname: &str) -> String {
    let scheme = if protocol == "https:" { "wss" } else { "ws" };
    let directory = &pathname[..pathname.rfind('/').map_or(0, |end| end + 1)];
    let directory = if directory.is_empty() { "/" } else { directory };
    format!("{scheme}://{host}{directory}ws")
}

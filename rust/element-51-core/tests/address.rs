//! Where the page finds its websocket.

use element_51_core::address::websocket_url;

#[test]
fn at_the_root_the_socket_is_at_slash_ws() {
    assert_eq!(
        websocket_url("http:", "127.0.0.1:8765", "/"),
        "ws://127.0.0.1:8765/ws"
    );
}

#[test]
fn under_a_prefix_the_socket_is_beside_the_page() {
    assert_eq!(
        websocket_url("http:", "localhost:8000", "/editor/"),
        "ws://localhost:8000/editor/ws"
    );
    assert_eq!(
        websocket_url("http:", "localhost:8000", "/a/b/"),
        "ws://localhost:8000/a/b/ws"
    );
}

#[test]
fn a_file_name_in_the_path_is_dropped() {
    assert_eq!(
        websocket_url("http:", "localhost:8000", "/editor/index.html"),
        "ws://localhost:8000/editor/ws"
    );
}

#[test]
fn a_secure_page_gets_a_secure_socket() {
    assert_eq!(
        websocket_url("https:", "example.org", "/tools/"),
        "wss://example.org/tools/ws"
    );
}

#[test]
fn an_empty_path_is_the_root() {
    assert_eq!(websocket_url("http:", "h", ""), "ws://h/ws");
}

use backend_service::route_request;

#[test]
fn health_endpoint_works() {
    let resp = route_request("GET /health HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert_eq!(resp.status_line, "HTTP/1.1 200 OK");
    assert!(resp.body.contains("ok"));
}

#[test]
fn devices_endpoint_returns_payload() {
    let resp = route_request("GET /api/v1/discovery/devices HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert_eq!(resp.status_line, "HTTP/1.1 200 OK");
    assert!(resp.body.contains("\"devices\""));
    assert!(resp.body.contains("peer-a"));
}

#[test]
fn unknown_route_returns_404() {
    let resp = route_request("GET /missing HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert_eq!(resp.status_line, "HTTP/1.1 404 Not Found");
}

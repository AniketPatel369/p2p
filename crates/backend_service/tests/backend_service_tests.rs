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
fn create_transfer_returns_queued_transfer() {
    let request = "POST /api/v1/transfers HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: 63\r\n\r\n{\"file_name\":\"demo.txt\",\"receiver_ids\":[\"peer-a\",\"peer-b\"]}";
    let resp = route_request(request);

    assert_eq!(resp.status_line, "HTTP/1.1 201 Created");
    assert!(resp.body.contains("\"status\":\"queued\""));
    assert!(resp.body.contains("\"transfer_id\":"));
}

#[test]
fn create_transfer_requires_receiver_ids() {
    let request = "POST /api/v1/transfers HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: 25\r\n\r\n{\"file_name\":\"demo.txt\"}";
    let resp = route_request(request);

    assert_eq!(resp.status_line, "HTTP/1.1 400 Bad Request");
    assert!(resp.body.contains("receiver_ids_required"));
}

#[test]
fn unknown_route_returns_404() {
    let resp = route_request("GET /missing HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert_eq!(resp.status_line, "HTTP/1.1 404 Not Found");
}

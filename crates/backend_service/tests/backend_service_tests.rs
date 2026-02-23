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
fn incoming_request_endpoint_returns_pending_request() {
    let resp = route_request("GET /api/v1/incoming-request HTTP/1.1\r\nHost: localhost\r\n\r\n");

    assert_eq!(resp.status_line, "HTTP/1.1 200 OK");
    assert!(resp.body.contains("request_id"));
    assert!(resp.body.contains("holiday_photos.zip"));
}

#[test]
fn incoming_request_decision_records_accept() {
    let request = "POST /api/v1/incoming-request/decision HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: 41\r\n\r\n{\"request_id\":7001,\"decision\":\"accepted\"}";
    let resp = route_request(request);

    assert_eq!(resp.status_line, "HTTP/1.1 200 OK");
    assert!(resp.body.contains("\"status\":\"recorded\""));
    assert!(resp.body.contains("\"decision\":\"accepted\""));
}

#[test]
fn incoming_request_decision_rejects_invalid_payload() {
    let request = "POST /api/v1/incoming-request/decision HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: 17\r\n\r\n{\"request_id\":0}";
    let resp = route_request(request);

    assert_eq!(resp.status_line, "HTTP/1.1 400 Bad Request");
    assert!(resp.body.contains("invalid_decision_payload"));
}

#[test]
fn transfer_progress_endpoint_advances_and_completes() {
    let progress_20 = route_request(
        "GET /api/v1/transfers/progress?transfer_id=2001&poll=1 HTTP/1.1\r\nHost: localhost\r\n\r\n",
    );
    assert_eq!(progress_20.status_line, "HTTP/1.1 200 OK");
    assert!(progress_20.body.contains("\"progress_percent\":20"));
    assert!(progress_20.body.contains("\"status\":\"in-progress\""));

    let progress_100 = route_request(
        "GET /api/v1/transfers/progress?transfer_id=2001&poll=5 HTTP/1.1\r\nHost: localhost\r\n\r\n",
    );
    assert!(progress_100.body.contains("\"progress_percent\":100"));
    assert!(progress_100.body.contains("\"status\":\"completed\""));
}

#[test]
fn transfer_progress_requires_transfer_id() {
    let resp =
        route_request("GET /api/v1/transfers/progress?poll=1 HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert_eq!(resp.status_line, "HTTP/1.1 400 Bad Request");
    assert!(resp.body.contains("transfer_id_required"));
}

#[test]
fn security_state_endpoint_returns_fingerprint_and_trust_state() {
    let resp = route_request("GET /api/v1/security/state HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert_eq!(resp.status_line, "HTTP/1.1 200 OK");
    assert!(resp.body.contains("local_fingerprint"));
    assert!(resp.body.contains("trust_state"));
}

#[test]
fn trust_state_save_accepts_trusted() {
    let request = "POST /api/v1/security/trust HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: 25\r\n\r\n{\"trust_state\":\"trusted\"}";
    let resp = route_request(request);
    assert_eq!(resp.status_line, "HTTP/1.1 200 OK");
    assert!(resp.body.contains("\"trust_state\":\"trusted\""));
}

#[test]
fn settings_get_returns_defaults() {
    let resp = route_request("GET /api/v1/settings HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert_eq!(resp.status_line, "HTTP/1.1 200 OK");
    assert!(resp.body.contains("\"lan_only\":true"));
    assert!(resp.body.contains("\"update_channel\":\"stable\""));
}

#[test]
fn settings_post_roundtrips_payload_values() {
    let request = "POST /api/v1/settings HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: 84\r\n\r\n{\"lan_only\":false,\"relay_enabled\":true,\"diagnostics_enabled\":true,\"update_channel\":\"beta\"}";
    let resp = route_request(request);
    assert_eq!(resp.status_line, "HTTP/1.1 200 OK");
    assert!(resp.body.contains("\"lan_only\":false"));
    assert!(resp.body.contains("\"relay_enabled\":true"));
    assert!(resp.body.contains("\"diagnostics_enabled\":true"));
    assert!(resp.body.contains("\"update_channel\":\"beta\""));
}

#[test]
fn unknown_route_returns_404() {
    let resp = route_request("GET /missing HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert_eq!(resp.status_line, "HTTP/1.1 404 Not Found");
}

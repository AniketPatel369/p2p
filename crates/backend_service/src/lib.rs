#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status_line: &'static str,
    pub content_type: &'static str,
    pub body: String,
}

impl HttpResponse {
    pub fn to_http_string(&self) -> String {
        format!(
            "{}\r\nContent-Type: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            self.status_line,
            self.content_type,
            self.body.as_bytes().len(),
            self.body
        )
    }
}

pub fn route_request(request: &str) -> HttpResponse {
    if request.starts_with("OPTIONS ") {
        return HttpResponse {
            status_line: "HTTP/1.1 204 No Content",
            content_type: "text/plain; charset=utf-8",
            body: String::new(),
        };
    }

    if request.starts_with("GET /health ") {
        return HttpResponse {
            status_line: "HTTP/1.1 200 OK",
            content_type: "application/json; charset=utf-8",
            body: "{\"status\":\"ok\"}".to_string(),
        };
    }

    if request.starts_with("GET /api/v1/discovery/devices ") {
        return HttpResponse {
            status_line: "HTTP/1.1 200 OK",
            content_type: "application/json; charset=utf-8",
            body: discovery_devices_json(),
        };
    }

    HttpResponse {
        status_line: "HTTP/1.1 404 Not Found",
        content_type: "application/json; charset=utf-8",
        body: "{\"error\":\"not_found\"}".to_string(),
    }
}

fn discovery_devices_json() -> String {
    "{\"devices\":[{\"id\":\"peer-a\",\"name\":\"Aarav iPhone\",\"addr\":\"192.168.1.12\",\"status\":\"online\"},{\"id\":\"peer-b\",\"name\":\"Meera MacBook\",\"addr\":\"192.168.1.34\",\"status\":\"busy\"},{\"id\":\"peer-c\",\"name\":\"Ravi Desktop\",\"addr\":\"192.168.1.55\",\"status\":\"offline\"}]}".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status_line: &'static str,
    pub content_type: &'static str,
    pub body: String,
}

impl HttpResponse {
    pub fn to_http_string(&self) -> String {
        format!(
            "{}\r\nContent-Type: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            self.status_line,
            self.content_type,
            self.body.len(),
            self.body
        )
    }
}

pub fn route_request(request: &str) -> HttpResponse {
    let (first_line, body) = split_request(request);

    if first_line.starts_with("OPTIONS ") {
        return HttpResponse {
            status_line: "HTTP/1.1 204 No Content",
            content_type: "text/plain; charset=utf-8",
            body: String::new(),
        };
    }

    if first_line.starts_with("GET /health ") {
        return HttpResponse {
            status_line: "HTTP/1.1 200 OK",
            content_type: "application/json; charset=utf-8",
            body: "{\"status\":\"ok\"}".to_string(),
        };
    }

    if first_line.starts_with("GET /api/v1/discovery/devices ") {
        return HttpResponse {
            status_line: "HTTP/1.1 200 OK",
            content_type: "application/json; charset=utf-8",
            body: discovery_devices_json(),
        };
    }

    if first_line.starts_with("POST /api/v1/transfers ") {
        return route_create_transfer(body);
    }

    HttpResponse {
        status_line: "HTTP/1.1 404 Not Found",
        content_type: "application/json; charset=utf-8",
        body: "{\"error\":\"not_found\"}".to_string(),
    }
}

fn route_create_transfer(body: &str) -> HttpResponse {
    let file_name =
        extract_json_string(body, "file_name").unwrap_or_else(|| "unknown.bin".to_string());
    let receiver_ids = extract_json_string_array(body, "receiver_ids").unwrap_or_default();

    if receiver_ids.is_empty() {
        return HttpResponse {
            status_line: "HTTP/1.1 400 Bad Request",
            content_type: "application/json; charset=utf-8",
            body: "{\"error\":\"receiver_ids_required\"}".to_string(),
        };
    }

    let transfer_id = 1_000 + file_name.len() as u64 + receiver_ids.len() as u64;
    let receivers_json = receiver_ids
        .iter()
        .map(|r| format!("\"{}\"", escape_json(r)))
        .collect::<Vec<_>>()
        .join(",");

    HttpResponse {
        status_line: "HTTP/1.1 201 Created",
        content_type: "application/json; charset=utf-8",
        body: format!(
            "{{\"transfer_id\":{},\"status\":\"queued\",\"file_name\":\"{}\",\"receiver_ids\":[{}]}}",
            transfer_id,
            escape_json(&file_name),
            receivers_json
        ),
    }
}

fn split_request(request: &str) -> (&str, &str) {
    let mut lines = request.lines();
    let first_line = lines.next().unwrap_or_default();

    if let Some((_, body)) = request.split_once("\r\n\r\n") {
        (first_line, body)
    } else if let Some((_, body)) = request.split_once("\n\n") {
        (first_line, body)
    } else {
        (first_line, "")
    }
}

fn extract_json_string(body: &str, key: &str) -> Option<String> {
    let marker = format!("\"{}\"", key);
    let idx = body.find(&marker)?;
    let after = &body[idx + marker.len()..];
    let colon = after.find(':')?;
    let after_colon = after[colon + 1..].trim_start();
    let first_quote = after_colon.find('"')?;
    let rest = &after_colon[first_quote + 1..];
    let end_quote = rest.find('"')?;
    Some(rest[..end_quote].to_string())
}

fn extract_json_string_array(body: &str, key: &str) -> Option<Vec<String>> {
    let marker = format!("\"{}\"", key);
    let idx = body.find(&marker)?;
    let after = &body[idx + marker.len()..];
    let colon = after.find(':')?;
    let after_colon = after[colon + 1..].trim_start();

    let open = after_colon.find('[')?;
    let close = after_colon[open + 1..].find(']')? + open + 1;
    let array_segment = &after_colon[open + 1..close];

    let mut values = Vec::new();
    for part in array_segment.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
            values.push(trimmed[1..trimmed.len() - 1].to_string());
        }
    }

    Some(values)
}

fn escape_json(input: &str) -> String {
    input.replace('"', "\\\"")
}

fn discovery_devices_json() -> String {
    "{\"devices\":[{\"id\":\"peer-a\",\"name\":\"Aarav iPhone\",\"addr\":\"192.168.1.12\",\"status\":\"online\"},{\"id\":\"peer-b\",\"name\":\"Meera MacBook\",\"addr\":\"192.168.1.34\",\"status\":\"busy\"},{\"id\":\"peer-c\",\"name\":\"Ravi Desktop\",\"addr\":\"192.168.1.55\",\"status\":\"offline\"}]}".to_string()
}

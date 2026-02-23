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

    if first_line.starts_with("GET /api/v1/transfers/progress?") {
        return route_transfer_progress(first_line);
    }

    if first_line.starts_with("GET /api/v1/incoming-request ") {
        return HttpResponse {
            status_line: "HTTP/1.1 200 OK",
            content_type: "application/json; charset=utf-8",
            body: "{\"request\":{\"request_id\":7001,\"from\":\"Aarav iPhone\",\"file_name\":\"holiday_photos.zip\",\"size\":\"128 MB\"}}".to_string(),
        };
    }

    if first_line.starts_with("POST /api/v1/incoming-request/decision ") {
        return route_incoming_decision(body);
    }

    if first_line.starts_with("GET /api/v1/security/state ") {
        return HttpResponse {
            status_line: "HTTP/1.1 200 OK",
            content_type: "application/json; charset=utf-8",
            body:
                "{\"local_fingerprint\":\"FA:13:7B:2C:90:AA:45:99\",\"trust_state\":\"unverified\"}"
                    .to_string(),
        };
    }

    if first_line.starts_with("POST /api/v1/security/trust ") {
        return route_security_trust(body);
    }

    if first_line.starts_with("GET /api/v1/settings ") {
        return HttpResponse {
            status_line: "HTTP/1.1 200 OK",
            content_type: "application/json; charset=utf-8",
            body: "{\"lan_only\":true,\"relay_enabled\":false,\"diagnostics_enabled\":false,\"update_channel\":\"stable\"}".to_string(),
        };
    }

    if first_line.starts_with("POST /api/v1/settings ") {
        return route_settings_save(body);
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

fn route_transfer_progress(first_line: &str) -> HttpResponse {
    let transfer_id = extract_query_u64(first_line, "transfer_id").unwrap_or(0);
    let poll = extract_query_u64(first_line, "poll").unwrap_or(0);

    if transfer_id == 0 {
        return HttpResponse {
            status_line: "HTTP/1.1 400 Bad Request",
            content_type: "application/json; charset=utf-8",
            body: "{\"error\":\"transfer_id_required\"}".to_string(),
        };
    }

    let progress = (poll.saturating_mul(20)).min(100);
    let status = if progress >= 100 {
        "completed"
    } else {
        "in-progress"
    };

    HttpResponse {
        status_line: "HTTP/1.1 200 OK",
        content_type: "application/json; charset=utf-8",
        body: format!(
            "{{\"transfer_id\":{},\"progress_percent\":{},\"status\":\"{}\"}}",
            transfer_id, progress, status
        ),
    }
}

fn route_incoming_decision(body: &str) -> HttpResponse {
    let request_id = extract_json_u64(body, "request_id").unwrap_or(0);
    let decision = extract_json_string(body, "decision").unwrap_or_default();

    if request_id == 0 || (decision != "accepted" && decision != "declined") {
        return HttpResponse {
            status_line: "HTTP/1.1 400 Bad Request",
            content_type: "application/json; charset=utf-8",
            body: "{\"error\":\"invalid_decision_payload\"}".to_string(),
        };
    }

    HttpResponse {
        status_line: "HTTP/1.1 200 OK",
        content_type: "application/json; charset=utf-8",
        body: format!(
            "{{\"request_id\":{},\"decision\":\"{}\",\"status\":\"recorded\"}}",
            request_id, decision
        ),
    }
}

fn route_security_trust(body: &str) -> HttpResponse {
    let trust_state = extract_json_string(body, "trust_state").unwrap_or_default();
    if trust_state != "trusted" && trust_state != "unverified" {
        return HttpResponse {
            status_line: "HTTP/1.1 400 Bad Request",
            content_type: "application/json; charset=utf-8",
            body: "{\"error\":\"invalid_trust_state\"}".to_string(),
        };
    }

    HttpResponse {
        status_line: "HTTP/1.1 200 OK",
        content_type: "application/json; charset=utf-8",
        body: format!(
            "{{\"trust_state\":\"{}\",\"status\":\"saved\"}}",
            trust_state
        ),
    }
}

fn route_settings_save(body: &str) -> HttpResponse {
    let lan_only = extract_json_bool(body, "lan_only").unwrap_or(true);
    let relay_enabled = extract_json_bool(body, "relay_enabled").unwrap_or(false);
    let diagnostics_enabled = extract_json_bool(body, "diagnostics_enabled").unwrap_or(false);
    let update_channel =
        extract_json_string(body, "update_channel").unwrap_or_else(|| "stable".to_string());

    let normalized_channel =
        if update_channel == "stable" || update_channel == "beta" || update_channel == "nightly" {
            update_channel
        } else {
            "stable".to_string()
        };

    HttpResponse {
        status_line: "HTTP/1.1 200 OK",
        content_type: "application/json; charset=utf-8",
        body: format!(
            "{{\"lan_only\":{},\"relay_enabled\":{},\"diagnostics_enabled\":{},\"update_channel\":\"{}\"}}",
            lan_only, relay_enabled, diagnostics_enabled, normalized_channel
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

fn extract_json_bool(body: &str, key: &str) -> Option<bool> {
    let marker = format!("\"{}\"", key);
    let idx = body.find(&marker)?;
    let after = &body[idx + marker.len()..];
    let colon = after.find(':')?;
    let after_colon = after[colon + 1..].trim_start();

    if after_colon.starts_with("true") {
        Some(true)
    } else if after_colon.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

fn extract_json_u64(body: &str, key: &str) -> Option<u64> {
    let marker = format!("\"{}\"", key);
    let idx = body.find(&marker)?;
    let after = &body[idx + marker.len()..];
    let colon = after.find(':')?;
    let after_colon = after[colon + 1..].trim_start();

    let digits = after_colon
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>();

    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
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

fn extract_query_u64(first_line: &str, key: &str) -> Option<u64> {
    let (_, rest) = first_line.split_once(' ')?;
    let (target, _) = rest.split_once(' ')?;
    let (_, query) = target.split_once('?')?;

    for pair in query.split('&') {
        let (k, v) = pair.split_once('=')?;
        if k == key {
            if let Ok(parsed) = v.parse::<u64>() {
                return Some(parsed);
            }
        }
    }

    None
}

fn escape_json(input: &str) -> String {
    input.replace('"', "\\\"")
}

fn discovery_devices_json() -> String {
    "{\"devices\":[{\"id\":\"peer-a\",\"name\":\"Aarav iPhone\",\"addr\":\"192.168.1.12\",\"status\":\"online\"},{\"id\":\"peer-b\",\"name\":\"Meera MacBook\",\"addr\":\"192.168.1.34\",\"status\":\"busy\"},{\"id\":\"peer-c\",\"name\":\"Ravi Desktop\",\"addr\":\"192.168.1.55\",\"status\":\"offline\"}]}".to_string()
}

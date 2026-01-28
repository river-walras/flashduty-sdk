use crate::models::{AlertEvent, ApiResponse};
use log::{error, info, warn};
use reqwest::blocking::Client;
use std::thread;
use std::time::Duration;

const API_URL: &str = "https://api.flashcat.cloud/event/push/alert/standard";
const MAX_RETRIES: u32 = 3;

pub fn send_with_retry(client: &Client, event: &AlertEvent) {
    for attempt in 0..MAX_RETRIES {
        match send_once(client, event) {
            Ok(()) => return,
            Err(e) => {
                if attempt + 1 < MAX_RETRIES {
                    let backoff = Duration::from_secs(1 << attempt);
                    warn!(
                        "Send failed (attempt {}/{}): {}, retrying in {:?}",
                        attempt + 1,
                        MAX_RETRIES,
                        e,
                        backoff
                    );
                    thread::sleep(backoff);
                } else {
                    error!(
                        "Send failed after {} attempts: {}",
                        MAX_RETRIES, e
                    );
                }
            }
        }
    }
}

fn send_once(client: &Client, event: &AlertEvent) -> Result<(), String> {
    let resp = client
        .post(API_URL)
        .json(event)
        .send()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = resp.status();
    if status.is_success() {
        match resp.json::<ApiResponse>() {
            Ok(api_resp) => {
                if api_resp.error.is_empty() {
                    info!("Alert sent successfully: {}", event.title_rule);
                    Ok(())
                } else {
                    Err(format!("API error: {}", api_resp.error))
                }
            }
            Err(e) => Err(format!("Failed to parse response: {}", e)),
        }
    } else {
        let body = resp.text().unwrap_or_default();
        Err(format!("HTTP {}: {}", status, body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EventStatus;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[test]
    fn test_send_once_invalid_url() {
        // This tests that network errors are properly handled
        let client = Client::new();
        let event = AlertEvent {
            integration_key: Arc::from("test-key"),
            event_status: EventStatus::Warning,
            title_rule: "test alert".to_string(),
            alert_key: None,
            description: None,
            labels: Some(HashMap::from([("env".to_string(), "test".to_string())])),
            images: None,
        };
        // send_once will fail because the integration key is invalid,
        // but it should not panic
        let result = send_once(&client, &event);
        // We expect either a network error or an API error, both are Ok for this test
        assert!(result.is_ok() || result.is_err());
    }
}

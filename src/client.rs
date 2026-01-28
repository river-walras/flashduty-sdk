use crate::models::{AlertEvent, EventStatus, Image};
use crate::sender::send_with_retry;
use crossbeam_channel::{Sender, unbounded};
use log::info;
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

pub struct FlashDutyClient {
    integration_key: Arc<str>,
    sender: Option<Sender<AlertEvent>>,
    handle: Option<JoinHandle<()>>,
    shutdown_flag: Arc<AtomicBool>,
}

impl FlashDutyClient {
    pub fn new(integration_key: String) -> Self {
        let integration_key: Arc<str> = Arc::from(integration_key);
        let (tx, rx) = unbounded::<AlertEvent>();
        let shutdown_flag = Arc::new(AtomicBool::new(false));

        let handle = thread::spawn(move || {
            let client = Client::new();
            while let Ok(event) = rx.recv() {
                send_with_retry(&client, &event);
            }
            info!("Background sender thread exiting, queue drained");
        });

        FlashDutyClient {
            integration_key,
            sender: Some(tx),
            handle: Some(handle),
            shutdown_flag,
        }
    }

    pub fn send_alert(
        &self,
        event_status: EventStatus,
        title_rule: String,
        alert_key: Option<String>,
        description: Option<String>,
        labels: Option<HashMap<String, String>>,
        images: Option<Vec<Image>>,
    ) {
        if self.shutdown_flag.load(Ordering::Relaxed) {
            log::warn!("Client already shut down, ignoring send_alert");
            return;
        }

        let event = AlertEvent {
            integration_key: Arc::clone(&self.integration_key),
            event_status,
            title_rule,
            alert_key,
            description,
            labels,
            images,
        };

        if let Some(ref tx) = self.sender {
            if let Err(e) = tx.send(event) {
                log::error!("Failed to enqueue alert: {}", e);
            }
        }
    }

    pub fn shutdown(&mut self) {
        if self.shutdown_flag.swap(true, Ordering::SeqCst) {
            return; // already shut down
        }

        // Drop sender to close the channel
        self.sender.take();

        // Wait for background thread to finish
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }

        info!("FlashDutyClient shut down");
    }
}

impl Drop for FlashDutyClient {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_create_and_shutdown() {
        let mut client = FlashDutyClient::new("test-key".to_string());
        // Enqueue an event (will fail to send but should not panic)
        client.send_alert(
            EventStatus::Warning,
            "test alert".to_string(),
            None,
            None,
            None,
            None,
        );
        client.shutdown();
        // Double shutdown should be safe
        client.shutdown();
    }

    #[test]
    fn test_send_after_shutdown_is_ignored() {
        let mut client = FlashDutyClient::new("test-key".to_string());
        client.shutdown();
        // This should not panic
        client.send_alert(
            EventStatus::Warning,
            "ignored alert".to_string(),
            None,
            None,
            None,
            None,
        );
    }
}

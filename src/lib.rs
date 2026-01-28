pub mod client;
pub mod models;
pub mod sender;

#[cfg(feature = "python")]
mod python {
    use crate::client::FlashDutyClient as RustClient;
    use crate::models::{EventStatus, Image};
    use pyo3::prelude::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    fn parse_event_status(s: &str) -> PyResult<EventStatus> {
        match s {
            "Ok" => Ok(EventStatus::Ok),
            "Info" => Ok(EventStatus::Info),
            "Warning" => Ok(EventStatus::Warning),
            "Critical" => Ok(EventStatus::Critical),
            _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Invalid event_status: '{}'. Must be one of: Ok, Info, Warning, Critical",
                s
            ))),
        }
    }

    #[pyclass(name = "FlashDutyClient")]
    struct PyFlashDutyClient {
        inner: Mutex<Option<RustClient>>,
    }

    #[pymethods]
    impl PyFlashDutyClient {
        #[new]
        fn new(integration_key: String) -> Self {
            PyFlashDutyClient {
                inner: Mutex::new(Some(RustClient::new(integration_key))),
            }
        }

        #[pyo3(signature = (
            event_status,
            title_rule,
            alert_key = None,
            description = None,
            labels = None,
            images = None,
        ))]
        fn send_alert(
            &self,
            event_status: &str,
            title_rule: String,
            alert_key: Option<String>,
            description: Option<String>,
            labels: Option<HashMap<String, String>>,
            images: Option<Vec<HashMap<String, String>>>,
        ) -> PyResult<()> {
            let status = parse_event_status(event_status)?;
            let images = images.map(|imgs| {
                imgs.into_iter()
                    .map(|m| Image {
                        src: m.get("src").cloned().unwrap_or_default(),
                        href: m.get("href").cloned(),
                        alt: m.get("alt").cloned(),
                    })
                    .collect()
            });

            let guard = self.inner.lock().map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!("Lock poisoned: {}", e))
            })?;
            if let Some(ref client) = *guard {
                client.send_alert(status, title_rule, alert_key, description, labels, images);
            } else {
                return Err(pyo3::exceptions::PyRuntimeError::new_err(
                    "Client already shut down",
                ));
            }
            Ok(())
        }

        fn shutdown(&self) -> PyResult<()> {
            let mut guard = self.inner.lock().map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!("Lock poisoned: {}", e))
            })?;
            if let Some(mut client) = guard.take() {
                client.shutdown();
            }
            Ok(())
        }

        fn __del__(&self) -> PyResult<()> {
            self.shutdown()
        }
    }

    #[pymodule]
    fn flashduty_sdk(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_class::<PyFlashDutyClient>()?;
        Ok(())
    }
}

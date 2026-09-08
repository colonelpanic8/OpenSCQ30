use std::sync::{Arc, Mutex};

use openscq30_lib::device::OpenSCQ30Device as LibOpenSCQ30Device;
use tokio::task::JoinHandle;

use crate::serializable;

#[derive(uniffi::Object)]
pub struct OpenSCQ30Device {
    pub inner: Arc<dyn LibOpenSCQ30Device + Send + Sync>,
    connection_status_callback: Arc<Mutex<Option<Arc<dyn ConnectionStatusCallback>>>>,
    connection_status_handle: JoinHandle<()>,
    watch_for_changes_callback: Arc<Mutex<Option<Arc<dyn NotificationCallback>>>>,
    watch_for_changes_handle: JoinHandle<()>,
    event_callback: Arc<Mutex<Option<Arc<dyn DeviceEventCallback>>>>,
    event_handle: Option<JoinHandle<()>>,
}

impl Drop for OpenSCQ30Device {
    fn drop(&mut self) {
        self.connection_status_handle.abort();
        self.watch_for_changes_handle.abort();
        if let Some(handle) = self.event_handle.take() {
            handle.abort();
        }
    }
}

impl OpenSCQ30Device {
    pub async fn new(inner: Arc<dyn LibOpenSCQ30Device + Send + Sync>) -> Self {
        let connection_status_callback: Arc<Mutex<Option<Arc<dyn ConnectionStatusCallback>>>> =
            Default::default();
        let connection_status_handle = {
            let connection_status_callback = connection_status_callback.clone();
            let inner = inner.clone();
            tokio::spawn(async move {
                loop {
                    if inner.connection_status().changed().await.is_err() {
                        break;
                    }
                    if let Some(callback) = connection_status_callback.lock().unwrap().as_ref() {
                        callback.on_change(serializable::ConnectionStatus(
                            inner.connection_status().borrow().to_owned(),
                        ));
                    }
                }
            })
        };
        let watch_for_changes_callback: Arc<Mutex<Option<Arc<dyn NotificationCallback>>>> =
            Default::default();
        let watch_for_changes_handle = {
            let watch_for_changes_callback = watch_for_changes_callback.clone();
            let inner = inner.clone();
            tokio::spawn(async move {
                let mut watcher = inner.watch_for_changes();
                loop {
                    if watcher.changed().await.is_err() {
                        break;
                    }
                    if let Some(callback) = watch_for_changes_callback.lock().unwrap().as_ref() {
                        callback.on_notify();
                    }
                }
            })
        };
        let event_callback: Arc<Mutex<Option<Arc<dyn DeviceEventCallback>>>> = Default::default();
        let event_handle = inner.subscribe_to_events().map(|mut events| {
            let callback = event_callback.clone();
            tokio::spawn(async move {
                loop {
                    match events.recv().await {
                        Ok(openscq30_lib::device::DeviceEvent::AssistantRequested) => {
                            let callback = callback.lock().unwrap().clone();
                            if let Some(callback) = callback {
                                callback.on_event("assistant-requested".to_owned());
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                            events = events.resubscribe();
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
            })
        });
        Self {
            inner,
            connection_status_callback,
            connection_status_handle,
            watch_for_changes_callback,
            watch_for_changes_handle,
            event_callback,
            event_handle,
        }
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl OpenSCQ30Device {
    pub fn set_connection_status_callback(&self, callback: Arc<dyn ConnectionStatusCallback>) {
        *self.connection_status_callback.lock().unwrap() = Some(callback);
    }

    fn set_watch_for_changes_callback(&self, callback: Arc<dyn NotificationCallback>) {
        *self.watch_for_changes_callback.lock().unwrap() = Some(callback);
    }

    pub fn set_device_event_callback(&self, callback: Arc<dyn DeviceEventCallback>) {
        *self.event_callback.lock().unwrap() = Some(callback);
    }

    pub fn model(&self) -> serializable::DeviceModel {
        serializable::DeviceModel(self.inner.model())
    }

    pub fn categories(&self) -> Vec<serializable::CategoryId> {
        self.inner
            .categories()
            .into_iter()
            .map(serializable::CategoryId)
            .collect()
    }

    pub fn settings_in_category(
        &self,
        category_id: serializable::CategoryId,
    ) -> Vec<serializable::SettingId> {
        self.inner
            .settings_in_category(&category_id.0)
            .into_iter()
            .map(serializable::SettingId)
            .collect()
    }

    pub fn setting(&self, setting_id: serializable::SettingId) -> Option<serializable::Setting> {
        self.inner.setting(&setting_id.0).map(serializable::Setting)
    }

    pub async fn set_setting_values(
        &self,
        setting_values: Vec<SettingIdValuePair>,
    ) -> Result<(), crate::OpenSCQ30Error> {
        self.inner
            .set_setting_values(
                setting_values
                    .into_iter()
                    .map(|pair| (pair.setting.0, pair.value.0))
                    .collect(),
            )
            .await
            .map_err(Into::into)
    }
}

#[derive(uniffi::Record)]
pub struct SettingIdValuePair {
    setting: serializable::SettingId,
    value: serializable::Value,
}

#[uniffi::export(with_foreign)]
pub trait ConnectionStatusCallback: Send + Sync {
    fn on_change(&self, connection_status: serializable::ConnectionStatus);
}

#[uniffi::export(with_foreign)]
pub trait NotificationCallback: Send + Sync {
    fn on_notify(&self);
}

#[uniffi::export(with_foreign)]
pub trait DeviceEventCallback: Send + Sync {
    fn on_event(&self, event: String);
}

#[cfg(test)]
mod event_tests {
    use super::*;
    use async_trait::async_trait;
    use openscq30_lib::{
        DeviceModel,
        connection::ConnectionStatus,
        device::{self, DeviceEvent},
        settings::{CategoryId, Setting, SettingId, Value},
    };
    use tokio::sync::{broadcast, mpsc, watch};

    struct EventDevice {
        events: broadcast::Sender<DeviceEvent>,
        changes: watch::Sender<()>,
        connection: watch::Sender<ConnectionStatus>,
    }

    #[async_trait]
    impl LibOpenSCQ30Device for EventDevice {
        fn subscribe_to_events(&self) -> Option<broadcast::Receiver<DeviceEvent>> {
            Some(self.events.subscribe())
        }
        fn connection_status(&self) -> watch::Receiver<ConnectionStatus> {
            self.connection.subscribe()
        }
        fn model(&self) -> DeviceModel {
            DeviceModel::SoundcoreDevelopment
        }
        fn categories(&self) -> Vec<CategoryId> {
            Vec::new()
        }
        fn settings_in_category(&self, _: &CategoryId) -> Vec<SettingId> {
            Vec::new()
        }
        fn setting(&self, _: &SettingId) -> Option<Setting> {
            None
        }
        fn watch_for_changes(&self) -> watch::Receiver<()> {
            self.changes.subscribe()
        }
        async fn set_setting_values(&self, _: Vec<(SettingId, Value)>) -> device::Result<()> {
            Ok(())
        }
    }

    struct Callback(mpsc::UnboundedSender<String>);
    impl DeviceEventCallback for Callback {
        fn on_event(&self, event: String) {
            let _ = self.0.send(event);
        }
    }

    #[tokio::test]
    async fn forwards_events_without_replaying_state_and_releases_subscription() {
        let inner = Arc::new(EventDevice {
            events: broadcast::channel(16).0,
            changes: watch::channel(()).0,
            connection: watch::channel(ConnectionStatus::Connected).0,
        });
        let device = OpenSCQ30Device::new(inner.clone()).await;
        inner.events.send(DeviceEvent::AssistantRequested).unwrap();
        tokio::task::yield_now().await;
        let (tx, mut rx) = mpsc::unbounded_channel();
        std::thread::scope(|scope| {
            scope
                .spawn(|| device.set_device_event_callback(Arc::new(Callback(tx))))
                .join()
                .unwrap();
        });
        inner.changes.send_replace(());
        tokio::task::yield_now().await;
        assert_eq!(rx.try_recv(), Err(mpsc::error::TryRecvError::Empty));
        inner.events.send(DeviceEvent::AssistantRequested).unwrap();
        assert_eq!(
            tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
                .await
                .unwrap()
                .as_deref(),
            Some("assistant-requested")
        );
        drop(device);
        tokio::task::yield_now().await;
        assert_eq!(inner.events.receiver_count(), 0);
    }
}

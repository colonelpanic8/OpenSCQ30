use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use macaddr::MacAddr6;
use tokio::{
    sync::{broadcast, watch},
    task::JoinHandle,
};

use crate::{
    api::{
        connection::{
            ConnectionDescriptor, ConnectionStatus, RfcommBackend, RfcommConnection,
            RfcommServiceSelectionStrategy,
        },
        device::{self, DeviceEvent, OpenSCQ30Device, OpenSCQ30DeviceRegistry},
        settings::{CategoryId, Setting, SettingId, Value},
    },
    devices::{
        DeviceModel,
        soundcore::{
            self,
            common::{
                modules::button_configuration::COMMON_ACTIONS_WITH_GAME_MODE,
                packet::{
                    self, PacketIOController,
                    outbound::{RequestState, ToPacket},
                },
            },
        },
    },
    i18n::fl,
};

use super::state::State;

pub struct D1203Registry {
    backend: Arc<dyn RfcommBackend + Send + Sync>,
}

impl D1203Registry {
    pub fn new(backend: Arc<dyn RfcommBackend + Send + Sync>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl OpenSCQ30DeviceRegistry for D1203Registry {
    async fn devices(&self) -> device::Result<Vec<ConnectionDescriptor>> {
        Ok(self.backend.devices().await?.into_iter().collect())
    }

    async fn connect(
        &self,
        address: MacAddr6,
    ) -> device::Result<Arc<dyn OpenSCQ30Device + Send + Sync>> {
        let connection = self
            .backend
            .connect(
                address,
                RfcommServiceSelectionStrategy::Dynamic(|uuids| {
                    uuids
                        .into_iter()
                        .find(soundcore::is_soundcore_vendor_rfcomm_uuid)
                        .unwrap_or(soundcore::RFCOMM_UUID)
                }),
            )
            .await?;
        Ok(Arc::new(D1203Device::new(connection).await?))
    }
}

struct Snapshot {
    state: State,
    events: VecDeque<String>,
    sequence: u64,
}

struct D1203Device {
    _packet_io: PacketIOController,
    connection: Arc<dyn RfcommConnection + Send + Sync>,
    snapshot: Arc<RwLock<Snapshot>>,
    changes: watch::Sender<()>,
    events: broadcast::Sender<DeviceEvent>,
    receiver: JoinHandle<()>,
}

impl Drop for D1203Device {
    fn drop(&mut self) {
        self.receiver.abort();
    }
}

impl D1203Device {
    async fn new(connection: Arc<dyn RfcommConnection + Send + Sync>) -> device::Result<Self> {
        let (packet_io, mut packets) =
            PacketIOController::new(connection.clone(), packet::ChecksumKind::Suffix).await?;
        let reply = packet_io
            .send_with_response(&RequestState.to_packet())
            .await?;
        let state = State::parse(&reply.body).map_err(device::Error::other)?;
        let snapshot = Arc::new(RwLock::new(Snapshot {
            state,
            events: VecDeque::new(),
            sequence: 0,
        }));
        let changes = watch::channel(()).0;
        let events = broadcast::channel(16).0;
        let receiver = tokio::spawn({
            let snapshot = snapshot.clone();
            let changes = changes.clone();
            let events = events.clone();
            async move {
                while let Some(packet) = packets.recv().await {
                    // These commands contain microphone audio, not button events.
                    if matches!(packet.command.0, [0x18, 0x01 | 0x04]) {
                        continue;
                    }
                    if packet.command.0 == [0x18, 0x03] {
                        let _ = events.send(DeviceEvent::AssistantRequested);
                    }
                    let mut snapshot = snapshot.write().unwrap();
                    if packet.command == RequestState::COMMAND {
                        match State::parse(&packet.body) {
                            Ok(state) => snapshot.state = state,
                            Err(error) => tracing::warn!("Ignoring invalid D1203 state: {error}"),
                        }
                    }
                    snapshot.sequence = snapshot.sequence.saturating_add(1);
                    let summary = format!(
                        "#{} {:02x}:{:02x} ({} bytes)",
                        snapshot.sequence,
                        packet.command.0[0],
                        packet.command.0[1],
                        packet.body.len()
                    );
                    tracing::info!("D1203 event {summary}");
                    snapshot.events.push_back(summary);
                    while snapshot.events.len() > 12 {
                        snapshot.events.pop_front();
                    }
                    changes.send_replace(());
                }
            }
        });
        Ok(Self {
            _packet_io: packet_io,
            connection,
            snapshot,
            changes,
            events,
            receiver,
        })
    }
}

const BUTTONS: [(SettingId, usize); 12] = [
    (SettingId::LeftSinglePress, 0),
    (SettingId::RightSinglePress, 1),
    (SettingId::LeftDoublePress, 2),
    (SettingId::RightDoublePress, 3),
    (SettingId::LeftTriplePress, 4),
    (SettingId::RightTriplePress, 5),
    (SettingId::LeftLongPress, 6),
    (SettingId::RightLongPress, 7),
    (SettingId::LeftSlideUp, 8),
    (SettingId::RightSlideUp, 9),
    (SettingId::LeftSlideDown, 10),
    (SettingId::RightSlideDown, 11),
];

fn action_name(id: u8) -> String {
    if let Some(action) = COMMON_ACTIONS_WITH_GAME_MODE
        .iter()
        .find(|action| action.id == id)
    {
        return (action.localized_name)();
    }
    match id {
        11 => fl!("d1203-anka"),
        15 => fl!("none"),
        _ => fl!("d1203-unknown-action", id = id),
    }
}

fn information(value: String) -> Setting {
    Setting::Information {
        translated_value: value.clone(),
        value,
    }
}

#[async_trait]
impl OpenSCQ30Device for D1203Device {
    fn subscribe_to_events(&self) -> Option<broadcast::Receiver<DeviceEvent>> {
        Some(self.events.subscribe())
    }

    fn connection_status(&self) -> watch::Receiver<ConnectionStatus> {
        self.connection.connection_status()
    }
    fn model(&self) -> DeviceModel {
        DeviceModel::SoundcoreD1203
    }
    fn categories(&self) -> Vec<CategoryId> {
        vec![
            CategoryId::DeviceInformation,
            CategoryId::ButtonConfiguration,
        ]
    }
    fn settings_in_category(&self, category: &CategoryId) -> Vec<SettingId> {
        match category {
            CategoryId::DeviceInformation => vec![
                SettingId::BatteryLevelLeft,
                SettingId::BatteryLevelRight,
                SettingId::CaseBatteryLevel,
                SettingId::FirmwareVersionLeft,
                SettingId::FirmwareVersionRight,
                SettingId::CaseFirmwareVersion,
                SettingId::SerialNumber,
                SettingId::TwsStatus,
                SettingId::DeviceEvents,
            ],
            CategoryId::ButtonConfiguration => BUTTONS.iter().map(|(id, _)| *id).collect(),
            _ => Vec::new(),
        }
    }
    fn setting(&self, id: &SettingId) -> Option<Setting> {
        let snapshot = self.snapshot.read().unwrap();
        let state = &snapshot.state;
        if let Some((_, index)) = BUTTONS.iter().find(|(button, _)| button == id) {
            return state.buttons[*index].map(|[both, single]| {
                information(fl!(
                    "d1203-button-modes",
                    both = action_name(both),
                    single = action_name(single)
                ))
            });
        }
        let value = match id {
            SettingId::BatteryLevelLeft => format!("{}%", state.battery_left),
            SettingId::BatteryLevelRight => format!("{}%", state.battery_right),
            SettingId::CaseBatteryLevel => format!("{}%", state.battery_case?),
            SettingId::FirmwareVersionLeft => state.firmware_left.clone(),
            SettingId::FirmwareVersionRight => state.firmware_right.clone(),
            SettingId::CaseFirmwareVersion => state.firmware_case.clone()?,
            SettingId::SerialNumber => state.serial.clone(),
            SettingId::TwsStatus => {
                if state.tws_connected {
                    fl!("yes")
                } else {
                    fl!("no")
                }
            }
            SettingId::DeviceEvents => {
                if snapshot.events.is_empty() {
                    fl!("d1203-waiting-for-events")
                } else {
                    snapshot
                        .events
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            }
            _ => return None,
        };
        Some(information(value))
    }
    fn watch_for_changes(&self) -> watch::Receiver<()> {
        self.changes.subscribe()
    }
    async fn set_setting_values(&self, values: Vec<(SettingId, Value)>) -> device::Result<()> {
        if values.is_empty() {
            return Ok(());
        }
        Err(device::Error::other(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "D1203 settings are read-only until write commands have been verified",
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::connection::test_stub::StubRfcommConnection;

    #[tokio::test]
    async fn retains_unsolicited_events_without_exposing_audio() {
        let (connection, inbound, mut outbound) = StubRfcommConnection::new();
        let init = tokio::spawn(D1203Device::new(Arc::new(connection)));
        assert_eq!(
            outbound.recv().await.unwrap(),
            RequestState.to_packet().bytes_with_checksum()
        );
        inbound
            .send(
                packet::Inbound::new(RequestState::COMMAND, include_bytes!("state.bin").to_vec())
                    .bytes_with_checksum(),
            )
            .await
            .unwrap();
        let device = init.await.unwrap().unwrap();
        let mut changes = device.watch_for_changes();
        let mut events = device.subscribe_to_events().unwrap();
        inbound
            .send(
                packet::Inbound::new(packet::Command([0x18, 0x04]), vec![10, 20, 30])
                    .bytes_with_checksum(),
            )
            .await
            .unwrap();
        inbound
            .send(
                packet::Inbound::new(packet::Command([0x18, 0x03]), vec![1]).bytes_with_checksum(),
            )
            .await
            .unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(1), changes.changed())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(events.try_recv().unwrap(), DeviceEvent::AssistantRequested);
        assert_eq!(
            events.try_recv(),
            Err(broadcast::error::TryRecvError::Empty)
        );
        {
            let snapshot = device.snapshot.read().unwrap();
            assert_eq!(snapshot.sequence, 1);
            assert_eq!(snapshot.events.front().unwrap(), "#1 18:03 (1 bytes)");
            assert_eq!(snapshot.state.battery_left, 60);
        }

        for body in [vec![0], vec![1]] {
            inbound
                .send(
                    packet::Inbound::new(packet::Command([0x18, 0x03]), body).bytes_with_checksum(),
                )
                .await
                .unwrap();
            assert_eq!(
                tokio::time::timeout(std::time::Duration::from_secs(1), events.recv())
                    .await
                    .unwrap()
                    .unwrap(),
                DeviceEvent::AssistantRequested,
            );
        }
    }
}

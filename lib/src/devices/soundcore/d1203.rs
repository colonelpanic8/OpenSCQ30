use std::{collections::HashMap, sync::Arc};

use crate::{
    api::connection::RfcommBackend,
    devices::{
        DeviceModel,
        soundcore::common::{demo::DemoConnectionRegistry, device::SoundcoreDeviceConfig, packet},
    },
    storage,
};

mod device;
mod state;

pub fn device_registry(
    backend: Arc<dyn RfcommBackend + Send + Sync>,
    _database: Arc<storage::OpenSCQ30Database>,
    _model: DeviceModel,
) -> device::D1203Registry {
    device::D1203Registry::new(backend)
}

pub fn demo_device_registry(
    _database: Arc<storage::OpenSCQ30Database>,
    model: DeviceModel,
) -> device::D1203Registry {
    device::D1203Registry::new(Arc::new(DemoConnectionRegistry::new(
        model,
        HashMap::from([(
            packet::Command([1, 1]),
            packet::Inbound::new(
                packet::Command([1, 1]),
                include_bytes!("d1203/state.bin").to_vec(),
            ),
        )]),
        SoundcoreDeviceConfig::default(),
    )))
}

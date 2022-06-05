use serde::{de::DeserializeOwned, Serialize};

pub trait PacketData: Serialize + DeserializeOwned {
    fn id() -> u8;
}

mod heartbeat;
pub use heartbeat::HeartbeatPacketData;

mod allocation;
pub use allocation::*;

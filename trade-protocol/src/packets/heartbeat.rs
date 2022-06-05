use serde::{Deserialize, Serialize};

use super::PacketData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartbeatPacketData {
    pub unix_ms: i64,
}

impl PacketData for HeartbeatPacketData {
    fn id() -> u8 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::HeartbeatPacketData;

    #[test]
    fn serialization_works() {
        let packet = HeartbeatPacketData { unix_ms: 123456789 };

        let obtained_bytes = bincode::serialize(&packet).unwrap();
        let expected_bytes = vec![
            21,  // 0001 0101
            205, // 1100 1101
            91,  // 0101 1011
            7,   // 0000 0111
            0,   // 0000 0000
            0,   // 0000 0000
            0,   // 0000 0000
            0,   // 0000 0000
        ];

        assert_eq!(expected_bytes, obtained_bytes);
    }

    #[test]
    fn deserialization_works() {
        let bytes = vec![
            21,  // 0001 0101
            205, // 1100 1101
            91,  // 0101 1011
            7,   // 0000 0111
            0,   // 0000 0000
            0,   // 0000 0000
            0,   // 0000 0000
            0,   // 0000 0000
        ];

        let obtained_packet: HeartbeatPacketData = bincode::deserialize(&bytes).unwrap();
        let expected_packet = HeartbeatPacketData { unix_ms: 123456789 };

        assert_eq!(expected_packet, obtained_packet);
    }
}

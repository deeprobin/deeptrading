use serde::{Deserialize, Serialize};

use super::PacketData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllocationRequestPacketData;

impl PacketData for AllocationRequestPacketData {
    fn id() -> u8 {
        1
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllocationResponsePacketData {
    pub symbol: Option<String>,
}

impl PacketData for AllocationResponsePacketData {
    fn id() -> u8 {
        2
    }
}

#[cfg(test)]
mod tests {
    use super::AllocationRequestPacketData;
    use super::AllocationResponsePacketData;

    #[test]
    fn serialization_works_req() {
        let packet = AllocationRequestPacketData;

        let obtained_bytes = bincode::serialize(&packet).unwrap();
        let expected_bytes: Vec<u8> = vec![];

        assert_eq!(expected_bytes, obtained_bytes);
    }

    #[test]
    fn deserialization_works_req() {
        let bytes = vec![];

        let obtained_packet: AllocationRequestPacketData = bincode::deserialize(&bytes).unwrap();
        let expected_packet = AllocationRequestPacketData;

        assert_eq!(expected_packet, obtained_packet);
    }
    #[test]
    fn serialization_works_resp() {
        let packet = AllocationResponsePacketData {
            symbol: Some("AAPL".to_string()),
        };

        let obtained_bytes = bincode::serialize(&packet).unwrap();
        let expected_bytes: Vec<u8> = vec![
            4, 0, 0, 0, 0, 0, 0, 0,  // Length
            65, // A
            65, // A
            80, // P
            76, // L
        ];

        assert_eq!(expected_bytes, obtained_bytes);
    }

    #[test]
    fn deserialization_works_resp() {
        let bytes = vec![
            4, 0, 0, 0, 0, 0, 0, 0,  // Length
            65, // A
            65, // A
            80, // P
            76, // L
        ];

        let obtained_packet: AllocationResponsePacketData = bincode::deserialize(&bytes).unwrap();
        let expected_packet = AllocationResponsePacketData {
            symbol: Some("AAPL".to_string()),
        };

        assert_eq!(expected_packet, obtained_packet);
    }
}

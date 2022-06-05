use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::packets::{
    AllocationRequestPacketData, AllocationResponsePacketData, HeartbeatPacketData, PacketData,
};

#[derive(Clone, Debug)]
pub struct EncodedPacket {
    pub packet_id: u8,
    pub identifier: u64,
    pub payload: Vec<u8>,
}

impl EncodedPacket {
    pub async fn write<W: AsyncWrite + Unpin>(
        &self,
        write: &mut W,
    ) -> Result<(), tokio::io::Error> {
        write.write_u8(self.packet_id).await?;
        write.write_u64(self.identifier).await?;

        // TODO: Check packet size
        write.write_u32(self.payload.len() as u32).await?;
        write.write_all(&self.payload).await?;
        Ok(())
    }
    pub async fn read<R: AsyncRead + Unpin>(read: &mut R) -> Result<Self, tokio::io::Error> {
        let packet_id = read.read_u8().await?;
        let identifier = read.read_u64().await?;
        let payload_size = read.read_u32().await?;

        // TODO: Check payload size (Allocating big payload is not good)
        let mut payload = vec![0; payload_size as usize];
        read.read_exact(&mut payload).await?;

        Ok(EncodedPacket {
            packet_id,
            identifier,
            payload,
        })
    }
}

#[derive(Debug)]
pub enum Packet {
    Heartbeat(HeartbeatPacketData),
    AllocationRequest(AllocationRequestPacketData),
    AllocationResponse(AllocationResponsePacketData),
}

impl Packet {
    pub fn id(&self) -> u8 {
        match self {
            Packet::Heartbeat(_) => HeartbeatPacketData::id(),
            Packet::AllocationRequest(_) => AllocationRequestPacketData::id(),
            Packet::AllocationResponse(_) => AllocationResponsePacketData::id(),
        }
    }
    pub fn decode(encoded: &EncodedPacket) -> Result<Packet, bincode::Error> {
        match encoded.packet_id {
            0 => Ok(Packet::Heartbeat(bincode::deserialize(&encoded.payload)?)),
            1 => Ok(Packet::AllocationRequest(bincode::deserialize(
                &encoded.payload,
            )?)),
            2 => Ok(Packet::AllocationResponse(bincode::deserialize(
                &encoded.payload,
            )?)),
            _ => Err(Box::new(bincode::ErrorKind::DeserializeAnyNotSupported)),
        }
    }
    pub fn encode(&self, identifier: u64) -> EncodedPacket {
        let packet_id = self.id();
        match self {
            Packet::Heartbeat(data) => EncodedPacket {
                identifier,
                packet_id,
                payload: bincode::serialize(data).unwrap(),
            },
            Packet::AllocationRequest(data) => EncodedPacket {
                identifier,
                packet_id,
                payload: bincode::serialize(data).unwrap(),
            },
            Packet::AllocationResponse(data) => EncodedPacket {
                identifier,
                packet_id,
                payload: bincode::serialize(data).unwrap(),
            },
        }
    }
}

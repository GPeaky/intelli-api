use prost::{bytes::BytesMut as ProstBytesMut, Message};

use crate::protos::packet_header::PacketType;

include!(concat!(env!("OUT_DIR"), "/protos.packet_header.rs"));

pub(crate) mod batched;
pub(crate) mod car_motion_data;
pub(crate) mod event_data;
pub(crate) mod final_classification;
pub(crate) mod participants;
pub(crate) mod session_data;
pub(crate) mod session_history;

/// Converts Rust types to protobuf messages.
pub trait ToProtoMessage: Sized {
    /// Associated protobuf message type.
    type ProtoType: Message;

    /// Converts to protobuf message.
    fn to_proto(&self) -> Option<Self::ProtoType>;

    /// Wraps protobuf message in PacketHeader.
    ///
    /// # Args
    /// - `packet_type`: PacketType for header
    ///
    /// # Returns
    /// PacketHeader with serialized message, or None.
    fn to_packet_header(&self, packet_type: PacketType) -> Option<PacketHeader> {
        let proto_data = self.to_proto()?;
        let mut payload = ProstBytesMut::with_capacity(proto_data.encoded_len());

        proto_data.encode_raw(&mut payload);

        Some(PacketHeader {
            r#type: packet_type.into(),
            payload: payload.freeze(),
        })
    }
}

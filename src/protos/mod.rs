use prost::{bytes::BytesMut as ProstBytesMut, Message};

use crate::structs::{get_f1_packet_type, F1PacketTypeTagged};

include!(concat!(env!("OUT_DIR"), "/protos.packet_header.rs"));

pub(crate) mod batched;
pub(crate) mod car_motion_data;
pub(crate) mod event_data;
pub(crate) mod final_classification;
pub(crate) mod participants;
pub(crate) mod session_data;
pub(crate) mod session_history;

/// Converts Rust types to protobuf messages.
pub trait ToProtoMessage: Sized + F1PacketTypeTagged {
    /// Associated protobuf message type.
    type ProtoType: Message;

    /// Converts to protobuf message.
    fn to_proto(&self) -> Option<Self::ProtoType>;

    /// Wraps protobuf message in PacketHeader.
    ///
    /// # Returns
    /// PacketHeader with serialized message, or None.
    fn to_packet_header(&self) -> Option<PacketHeader> {
        let packet_type = get_f1_packet_type::<Self>()?;
        let proto_data = self.to_proto()?;
        let mut payload = ProstBytesMut::with_capacity(proto_data.encoded_len());

        proto_data.encode_raw(&mut payload);

        Some(PacketHeader {
            r#type: packet_type as i32,
            payload: payload.freeze(),
        })
    }
}

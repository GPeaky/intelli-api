use prost::Message;

use crate::protos::packet_header::PacketType;

include!(concat!(env!("OUT_DIR"), "/protos.packet_header.rs"));

pub(crate) mod batched;
pub(crate) mod car_motion_data;
pub(crate) mod event_data;
pub(crate) mod final_classification;
pub(crate) mod participants;
pub(crate) mod session_data;
pub(crate) mod session_history;

/// Converts Rust data structures to protobuf messages.
///
/// This trait defines how to transform a Rust type into its corresponding protobuf message for serialization.
pub trait ToProtoMessage: Sized {
    /// The associated protobuf message type.
    type ProtoType: Message;

    /// Converts the Rust type to its protobuf message equivalent.
    fn to_proto(&self) -> Option<Self::ProtoType>;

    /// Converts the type to a `PacketHeader` containing the serialized protobuf message.
    ///
    /// Uses `to_proto` for conversion, then wraps the serialized message in a `PacketHeader` for transmission.
    ///
    /// # Parameters
    /// - `packet_type`: The packet type used in the `PacketHeader`.
    ///
    /// # Returns
    /// A `PacketHeader` with the serialized protobuf message, or `None` if the data is not important.
    fn convert(&self, packet_type: PacketType) -> Option<PacketHeader> {
        let proto_data = self.to_proto()?;
        let proto_data: Vec<u8> = proto_data.encode_to_vec();

        Some(PacketHeader {
            r#type: packet_type.into(),
            payload: proto_data,
        })
    }
}

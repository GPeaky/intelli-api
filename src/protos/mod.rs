include!(concat!(env!("OUT_DIR"), "/packet_header_generated.rs"));

pub(crate) mod car_motion_data;
pub(crate) mod event_data;
pub(crate) mod final_classification;
pub(crate) mod participants;
pub(crate) mod session_data;
pub(crate) mod session_history;

pub use protos::packet_header::PacketType;

pub trait ToFlatBufferMessage {
    fn to_flatbuffer(self) -> Vec<u8>;

    fn convert_and_encode(self, packet_type: protos::packet_header::PacketType) -> Vec<u8>
    where
        Self: Sized,
    {
        let mut builder = flatbuffers::FlatBufferBuilder::new();

        let flatbuffer_data = self.to_flatbuffer();
        let payload = builder.create_vector(&flatbuffer_data[..]);

        let header = protos::packet_header::PacketHeader::create(
            &mut builder,
            &protos::packet_header::PacketHeaderArgs {
                type_: packet_type,
                payload: Some(payload),
            },
        );

        builder.finish(header, None);

        builder.finished_data().to_vec()
    }
}

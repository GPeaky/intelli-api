fn main() {
    let mut config = prost_build::Config::new();

    config.bytes(["."]);

    config
        .compile_protos(
            &[
                "protos/car_motion.proto",
                "protos/event_data.proto",
                "protos/final_classification.proto",
                "protos/participants.proto",
                "protos/session_data.proto",
                "protos/session_history.proto",
                "protos/packet_header.proto",
            ],
            &["protos/"],
        )
        .unwrap();
}

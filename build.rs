use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let generated_dir = Path::new(&out_dir).join("generated");

    flatc_rust::run(flatc_rust::Args {
        inputs: &[
            Path::new("protos/car_motion.fbs"),
            Path::new("protos/event_data.fbs"),
            Path::new("protos/final_classification.fbs"),
            Path::new("protos/participants.fbs"),
            Path::new("protos/packet_header.fbs"),
            Path::new("protos/session_data.fbs"),
            Path::new("protos/session_history.fbs"),
        ],
        out_dir: &generated_dir,
        ..Default::default()
    })
    .unwrap();
}

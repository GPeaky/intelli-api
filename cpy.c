use std::env;
use std::path::Path;

fn main() {
    // Obtiene el directorio de salida del build de Cargo
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR no está definido");

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
        out_dir: Path::new(&out_dir),
        ..Default::default()
    })
    .expect("La compilación del esquema de FlatBuffers falló");
}

use ntex::web::{HttpResponse, Responder};
use serde::Serialize;

#[derive(Serialize)]
struct Release {
    version: String,
    url: String,
    signature: String,
    pub_date: String, // RFC 3339
}

// TODO: Return the latest release from GitHub of the Intelli App
pub async fn latest_release() -> impl Responder {
    let latest_release = Release {
        version: "0.1.0".to_string(),
        url: "".to_string(),
        signature: "".to_string(),
        pub_date: "".to_string(),
    };

    HttpResponse::Ok().json(&latest_release)
}

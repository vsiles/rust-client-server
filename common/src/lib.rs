use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Payload {
    pub addr: String,
    pub message: String,
}

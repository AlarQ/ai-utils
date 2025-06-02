use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Base64Image {
    pub name: String,
    pub base64: String,
}

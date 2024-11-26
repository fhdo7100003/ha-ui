#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct DeviceName(String);

impl DeviceName {
    pub fn new(name: String) -> Self {
        Self(name)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

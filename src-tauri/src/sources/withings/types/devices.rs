use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DeviceBody {
    #[serde(default)]
    pub devices: Vec<DeviceEntry>,
}

#[derive(Debug, Deserialize)]
pub struct DeviceEntry {
    pub deviceid: Option<String>,
    #[serde(rename = "type")]
    pub device_type: Option<String>,
    pub model: Option<String>,
    pub model_id: Option<i64>,
    pub battery: Option<String>,
    #[allow(dead_code)]
    pub timezone: Option<String>,
    pub last_session_date: Option<i64>,
    pub first_session_date: Option<i64>,
}

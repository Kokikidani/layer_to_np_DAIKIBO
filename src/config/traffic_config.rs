use serde_derive::{ Deserialize, Serialize };

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrafficConfig {
    pub distribution_filepath: String,
    pub path_num: usize,
}

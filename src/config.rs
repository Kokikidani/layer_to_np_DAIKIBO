use serde_derive::{ Deserialize, Serialize };

use crate::utils;

mod debug_config;
mod network_config;
mod policy_config;
mod simulation_config;
mod traffic_config;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub simulation: simulation_config::SimulationConfig,
    pub debug: debug_config::DebugConfig,
    pub network: network_config::NetworkConfig,
    pub policy: policy_config::PolicyConfig,
    pub traffic: traffic_config::TrafficConfig,
}

impl Config {
    /// Config構造体を作成する
    /// toml形式で書くこと．
    pub fn new(file_name: &str) -> Config {
        // configファイルを文字列として読込
        match utils::read_file(file_name) {
            Ok(contents) => {
                // 文字列をTOMLファイルとして読込
                match toml::from_str(&contents) {
                    Ok(config) => config,
                    Err(_) => panic!("TOMLファイルのパースに失敗しました。"),
                }
            }
            Err(_) => panic!("ファイルの読込に失敗しました。"),
        }
    }
}
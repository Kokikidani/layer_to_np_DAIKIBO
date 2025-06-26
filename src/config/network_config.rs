use serde_derive::{ Deserialize, Serialize };

#[derive(Debug, Deserialize, Serialize, Clone)]
/// ネットワーク関連の設定
pub struct NetworkConfig {
    /// 対象物理トポロジ
    pub topology: String,
    /// WSSのサイズ、1xM
    pub wss_m: usize,
    /// ループ時ファイバ増加率判定値
    pub fiber_increase_rate_limit: f64,
    /// 設計モード
    pub design_mode: String,
    /// ノード構成
    pub node_configuration: String,
    /// 改造コンフィグ
    pub modification_config_filepath: String,
    /// ファイバをまとめるかどうか
    pub fiber_unification: bool
}

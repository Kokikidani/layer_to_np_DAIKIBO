use serde_derive::{ Deserialize, Serialize };

#[derive(Debug, Deserialize, Serialize, Clone)]
/// シミュレーション関連の設定
pub struct SimulationConfig {
    /// トラフィック強度
    pub traffic_intensity: f64,
    /// ランダムシード
    pub random_seed: u64,
    /// 統計情報出力先フォルダ
    pub outdir: String,
    pub pythonexe_path: String
}

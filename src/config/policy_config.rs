use serde_derive::{ Deserialize, Serialize };

#[derive(Debug, Deserialize, Serialize, Clone)]
/// ルーティングポリシー関連の設定
pub struct PolicyConfig {
    /// ルーティングポリシー (FF, RD)
    pub routing_policy: String,
}

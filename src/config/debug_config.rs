use serde_derive::{ Deserialize, Serialize };

#[derive(Debug, Deserialize, Serialize, Clone)]
/// デバッガ関連の設定
pub struct DebugConfig {
    /// パス需要の割当/再割り当てを出力するか
    pub log_demand_assign: bool,
    /// ファイバの増設を出力するか
    pub log_fiber_expand: bool,
    /// ファイバの削除を出力するか
    pub log_fiber_remove: bool,
    /// バイパスの設立・削除を出力するか
    pub log_bypass: bool,
    /// 各種統計情報を出力するか
    pub log_analysis: bool,
    /// タブーリストへの追加を出力するか
    pub log_taboo: bool,
    /// ステップ時にstate_matrixを出力するか
    pub log_state_matrix: bool,
}

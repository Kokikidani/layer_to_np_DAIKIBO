use crate::{ config::Config, network::{ Network, XCType }, np_core::StateMatrix, topology::Topology, Edge, SLOT };

mod designer;
pub mod expander;
pub mod output;
mod pathfinder;

pub mod dynamic;

pub mod ctrl_utils;

fn get_expand_edges(network: &Network, node_route: &[usize]) -> Vec<Edge> {
    // ノードルート->エッジルート
    let edge_route = {
        let mut out = vec![];

        for edge in node_route.windows(2).collect::<Vec<_>>() {
            out.push(Edge::new(edge[0], edge[1]));
        }

        out
    };

    // 各スロットでこのルートを使用する際，増設が必要となるファイバの数
    let mut expand_count = [0; SLOT];

    for &edge in &edge_route {
        // エッジ上で，各ファイバを探索したときに，スロットが空いているかどうか．
        // 0: 空いている 1: 埋まっている
        let mut state_matrix_flag = StateMatrix::new_fulfilled();

        for fiber_id in &network.get_fiber_id_on_edge_partial(&edge) {
            let fiber = network.get_fiber_by_id(fiber_id);
            if network.get_fiber_sd_xc_type(fiber) == [XCType::Wxc, XCType::Wxc] {
                state_matrix_flag &= fiber.state_matrixes[0];
            } else {
                continue;
            }
        }

        // エッジ上での結果を反映
        // スロットが埋まっていれば，そのスロットを選択したときに増設が必要となる
        for slot in 0..SLOT {
            expand_count[slot] += state_matrix_flag[slot] as usize;
        }
    }

    // ファイバ増設数が最も小さくなるような組み合わせを返す
    
    let (target_slot, _expand_count) = expand_count
        .iter()
        .enumerate()
        .min_by_key(|(_, &value)| value)
        .unwrap();

    let mut out = vec![];

    // 各エッジごとに空きスロットがあるか検索，なければ増設対象としてoutへpush
    for edge in edge_route {
        // 空きスロットはないものとして仮定
        let mut empty = false;

        // 各ファイバを探索
        for fiber_id in &network.get_fiber_id_on_edge_partial(&edge) {
            let fiber = network.get_fiber_by_id(fiber_id);
            if
                network.get_fiber_sd_xc_type(fiber) == [XCType::Wxc, XCType::Wxc] &&
                !fiber.state_matrixes[0][target_slot]
            {
                // 空きがあった
                empty = true;
                break;
            }
        }
        if !empty {
            // 空きがなかったら増設対象とする
            out.push(edge);
        }
    }

    out
}

pub fn main(config: &Config) -> (Network, Topology, String) {

    let xc_types = match config.network.node_configuration.to_uppercase().as_str() {
        "FXC"  => [XCType::Wxc, XCType::Fxc],
        "SXC"  => [XCType::Wxc, XCType::Sxc],
        "WBXC" => [XCType::Wxc, XCType::Wbxc],
        // "FXC-SXC" => [XCType::Wxc, XCType::Fxc, XCType::Sxc],
        _ => unimplemented!()
    };

    match config.network.design_mode.to_uppercase().as_str() {
        "BEST" | "best" => designer::iterative_designer::best_main(config, &xc_types),
        "SINGLE" | "single" | "once" | "ONCE" => designer::main(config, &xc_types),
        "WBXC" | "wbxc" => designer::wxc_wbxc_designer::main(config),
        "AVERAGE" | "average" => designer::iterative_designer::average_main(config),
        _ => panic!("Invalid `design_mode`"),
    }
}

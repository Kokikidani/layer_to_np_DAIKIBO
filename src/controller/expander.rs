use std::cmp::Reverse;

use fxhash::FxHashMap;

use crate::{
    config::Config,
    demand::Demand,
    network::{CoreIndex, Fiber, FiberID, Network, PortID, XCType},
    np_core::parameters::{MAX_BYPASS_LEN, MIN_BYPASS_LEN},
    utils::enumerate_subsequences,
    Edge, SD,
};

mod expand_sxc;
pub use expand_sxc::expand_sxc_fibers;
pub use expand_sxc::get_min_expand_route_cand;

mod expand_fxc;
pub use expand_fxc::expand_fxc_fibers;

mod expand_wxc;
pub use expand_wxc::expand_wxc_fibers;

mod expand_wbxc;
pub use expand_wbxc::expand_wbxc_fibers;

pub fn remove_fibers_by_edges(
    config: &Config,
    network: &mut Network,
    target: &[Edge],
) -> Vec<FiberID> {
    let mut removed_fiber_ids = Vec::new();

    for edge in target {
        let fiber_ids_on_edge = network.get_fiber_id_on_edge(edge);
        if let Some(first_w2w_fiber) = fiber_ids_on_edge
            .iter()
            .find(|x| network.get_fiber_sd_xc_type_by_id(x) == [XCType::Wxc, XCType::Wxc])
        {
            network.delete_fiber(config, first_w2w_fiber);
            removed_fiber_ids.push(*first_w2w_fiber);
        }
    }

    removed_fiber_ids
}
pub fn generate_new_fiber(
    network: &mut Network,
    edge: &Edge,
    src_type: XCType,
    dst_type: XCType,
) -> Fiber {
    let src_xc = network.get_xc_mut_on_node(edge.src.into(), &src_type);
    let src_xc_id = src_xc.id;
    let src_device = src_xc.generate_new_device(false);
    network.regist_port_id(&src_device, &src_xc_id);

    let dst_xc = network.get_xc_mut_on_node(edge.dst.into(), &dst_type);
    let dst_xc_id = dst_xc.id;
    let dst_device = dst_xc.generate_new_device(true);
    network.regist_port_id(&dst_device, &dst_xc_id);

    Fiber::new_scf(edge, src_device, dst_device, [src_type, dst_type])
}

fn generate_new_mc_fiber(
    network: &mut Network,
    edge: &Edge,
    src_type: XCType,
    dst_type: XCType,
) -> Fiber {
    let src_xc = network.get_xc_mut_on_node(edge.src.into(), &src_type);
    let src_xc_id = src_xc.id;
    let src_port_ids: Vec<PortID> = CoreIndex::iter()
        .iter()
        .map(|_| src_xc.generate_new_device(false))
        .collect();
    for src_device in src_port_ids.iter() {
        network.regist_port_id(src_device, &src_xc_id);
    }

    let dst_xc = network.get_xc_mut_on_node(edge.dst.into(), &dst_type);
    let dst_xc_id = dst_xc.id;
    let dst_port_ids: Vec<PortID> = CoreIndex::iter()
        .iter()
        .map(|_| dst_xc.generate_new_device(true))
        .collect();
    for dst_device in dst_port_ids.iter() {
        network.regist_port_id(dst_device, &dst_xc_id);
    }

    Fiber::new_mcf(edge, src_port_ids, dst_port_ids, [src_type, dst_type])
}

/// 二層まで対応，三層以上の場合，トップレイヤを始端・終端とするバイパスのみ認める
pub fn find_frequently_emerge_sub_routes_sd_with_xc_types(
    network: &Network,
    demand_list: &[Demand],
    taboo_list: &[SD],
    xc_types: &[XCType],
) -> Option<SD> {
    // SDの出現回数を記録
    let mut counter: FxHashMap<SD, usize> = FxHashMap::default();

    // for demand in demand_list {
    //     println!("Demand: {:?}", demand);
    //     if demand.fiber_ids.is_empty() {
    //         println!("Skipping demand with empty fiber_ids.");
    //     }
    // }

    // 全ての`Demand`に対して，`fiber_route`から`edge_route`を取得，
    // 連続部分列を列挙し，カウンタに追加する
    for demand in demand_list {
        if !demand.fiber_ids.is_empty() {
            let sub_routes =
                enumerate_subsequences(&demand.fiber_ids, MIN_BYPASS_LEN, Some(MAX_BYPASS_LEN));

            for sub_route in sub_routes {
                // `sub_route`の始端/終端XCを確認する
                // バイパス区間の始端/終端になることができない組み合わせであれば，カウントしない
                let first_fiber = network.get_fiber_by_id(sub_route.first().unwrap());
                let last_fiber = network.get_fiber_by_id(sub_route.last().unwrap());

                let first_xc_type = network.get_fiber_sd_xc_type(first_fiber)[0];
                let last_xc_type = network.get_fiber_sd_xc_type(last_fiber)[1];

                // バイパス区間の始端/終端になれるのは，最上部のレイヤのみ，とする
                let top_layer_xc_type = *xc_types.first().unwrap();
                if first_xc_type == top_layer_xc_type && last_xc_type == top_layer_xc_type {
                    // 既にバイパスとなっているファイバの組み合わせではないか
                    // 既にバイパスとなっている場合，全区間を通したtop_layer_portの数は2になる
                    let mut top_layer_port_count = 1;
                    for fiber_id in &sub_route {
                        if network.get_fiber_sd_xc_type_by_id(fiber_id)[1] == top_layer_xc_type {
                            top_layer_port_count += 1;

                            // 既に数が2を超えている場合，バイパスとなっている可能性はゼロ
                            if top_layer_port_count > 2 {
                                break;
                            }
                        }
                    }

                    // 既にバイパスとなっているため，次の`sub_route`へ
                    if top_layer_port_count == 2 {
                        continue;
                    }

                    // この区間のSDを獲得
                    let sd = SD::new_from_nodes(first_fiber.edge.src, last_fiber.edge.dst);

                    // この区間のカウンタを+1
                    let entry = counter.entry(sd).or_insert(0);
                    *entry += 1;
                }
            }
        }
    }

    let mut counter_vec: Vec<(SD, usize)> = counter
        .iter()
        .map(|(x_sd, x_count)| (*x_sd, *x_count))
        .collect();
    counter_vec.sort_by_key(|(x_sd, _x_count)| *x_sd);
    counter_vec.sort_by_key(|(_x_sd, x_count)| Reverse(*x_count));

    for (max_sd, _max_count) in counter_vec {
        if taboo_list.contains(&max_sd) {
            continue;
        }
        return Some(max_sd);
    }

    // while !counter.is_empty() {
    //     // 最大値をとるSDを取得
    //     let (max_sd, _max_count) = counter.iter().max_by_key(|(&_sd, &count)| count).unwrap();
    //     // タブーリストを検索，含まれていればスキップ
    //     if taboo_list.contains(max_sd) {
    //         let remove_sd = *max_sd;
    //         counter.remove_entry(&remove_sd);
    //         continue;
    //     }
    //     // 含まれていなければ，リターン
    //     return Some(*max_sd)
    // }

    // 全てのルートがタブーリストに入れられてしまった
    None
}
pub fn find_emerge_sub_routes_sd_with_xc_types_with_len(
    network: &Network,
    demand_list: &[Demand],
    taboo_list: &[SD],
    xc_types: &[XCType],
    bypass_len: usize,
) -> Vec<SD> {
    let mut counter: FxHashMap<SD, usize> = FxHashMap::default();

    for demand in demand_list {
        if !demand.fiber_ids.is_empty() {
            let sub_routes =
                enumerate_subsequences(&demand.fiber_ids, bypass_len, Some(bypass_len));

            for sub_route in sub_routes {
                let first_fiber = network.get_fiber_by_id(sub_route.first().unwrap());
                let last_fiber = network.get_fiber_by_id(sub_route.last().unwrap());

                let first_xc_type = network.get_fiber_sd_xc_type(first_fiber)[0];
                let last_xc_type = network.get_fiber_sd_xc_type(last_fiber)[1];

                let top_layer_xc_type = *xc_types.first().unwrap();
                if first_xc_type == top_layer_xc_type && last_xc_type == top_layer_xc_type {
                    let mut top_layer_port_count = 1;
                    for fiber_id in &sub_route {
                        if network.get_fiber_sd_xc_type_by_id(fiber_id)[1] == top_layer_xc_type {
                            top_layer_port_count += 1;
                            if top_layer_port_count > 2 {
                                break;
                            }
                        }
                    }

                    if top_layer_port_count == 2 {
                        continue;
                    }

                    let sd = SD::new_from_nodes(first_fiber.edge.src, last_fiber.edge.dst);
                    let entry = counter.entry(sd).or_insert(0);
                    *entry += 1;
                }
            }
        }
    }

    let mut counter_vec: Vec<(SD, usize)> = counter.into_iter().collect();

    // 出現頻度で降順ソート
    counter_vec.sort_by_key(|(_sd, count)| Reverse(*count));

    // タブーリストに含まれていないSDだけを返す
    let result: Vec<SD> = counter_vec
        .into_iter()
        .filter(|(sd, _)| !taboo_list.contains(sd))
        .map(|(sd, _)| sd)
        .collect();

    result
}

pub fn expand_fibers_with_xc_types(
    config: &Config,
    network: &mut Network,
    target_edges: &[Edge],
    xc_types: &[XCType],
    all_installed_edge: &[Vec<Edge>],
) -> (Vec<Fiber>, Vec<(Edge, XCType, XCType)>) {
    match xc_types {
        //[XCType::Wxc, XCType::Wbxc] => expand_wbxc_fibers(config, network, target_edges),
        [XCType::Wxc, XCType::Fxc] => {
            expand_fxc_fibers(config, network, target_edges, all_installed_edge)
        }
        [XCType::Wxc, XCType::Added_Wxc, XCType::Fxc] => {
            expand_fxc_fibers(config, network, target_edges, all_installed_edge)
        }
        //[XCType::Wxc, XCType::Sxc] => expand_sxc_fibers(config, network, target_edges),
        [XCType::Wbxc, XCType::Fxc] | [XCType::Wbxc, XCType::Sxc] | [XCType::Fxc, XCType::Sxc] => {
            unimplemented!()
        }
        [XCType::Wbxc, XCType::Wxc]
        | [XCType::Fxc, XCType::Wxc]
        | [XCType::Fxc, XCType::Wbxc]
        | [XCType::Sxc, XCType::Wxc]
        | [XCType::Sxc, XCType::Wbxc]
        | [XCType::Sxc, XCType::Fxc] => panic!("Ordering Error"),
        [XCType::Wxc, XCType::Wxc]
        | [XCType::Wbxc, XCType::Wbxc]
        | [XCType::Fxc, XCType::Fxc]
        | [XCType::Sxc, XCType::Sxc] => panic!("XCTypes Error"),
        [XCType::Wbxc, XCType::Added_Wxc] => todo!(),
        [XCType::Fxc, XCType::Added_Wxc] => todo!(),
        [XCType::Sxc, XCType::Added_Wxc] => todo!(),
        [XCType::Added_Wxc, XCType::Wxc] => todo!(),
        [XCType::Added_Wxc, XCType::Wbxc] => todo!(),
        [XCType::Added_Wxc, XCType::Fxc] => todo!(),
        [XCType::Added_Wxc, XCType::Sxc] => todo!(),
        [XCType::Added_Wxc, XCType::Added_Wxc] => todo!(),
        _ => panic!("Invalid combination of XC types"),
    }
}

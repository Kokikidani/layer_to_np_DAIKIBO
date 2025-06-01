use crate::{
    config::Config,
    debugger,
    network::{Fiber, Network, XCType},
    Edge,
};

use super::generate_new_fiber;

/// FXCバイパスを新設する
/// FXCバイパスを新設する
pub fn expand_fxc_fibers_install_edges(
    config: &Config,
    network: &mut Network,
    target_edges: &[Edge],
    all_installed_edges: &mut Vec<Vec<Edge>>, // ← mutable参照に変更
) -> (Vec<Fiber>, Vec<(Edge, XCType, XCType)>) {
    if target_edges.len() < 2 {
        panic!(
            "target_edges: {:?} should be longer than two.",
            target_edges
        );
    }

    //all_installed_edges.retain(|installed| !is_subsequence(installed, target_edges));

    let mut fibers: Vec<Fiber> = vec![];
    let mut edge_type_tuples: Vec<(Edge, XCType, XCType)> = vec![];

    // 最初のファイバ (WXC → FXC)
    let first_edge = target_edges.first().unwrap();
    let first_fiber = generate_new_fiber(network, first_edge, XCType::Wxc, XCType::Fxc);
    let mut prev_dst_device_id = first_fiber.dst_port_ids.clone();
    edge_type_tuples.push((*first_edge, XCType::Wxc, XCType::Fxc));
    fibers.push(first_fiber);

    // 中間ファイバ群 (FXC → FXC)
    for (index, edge) in target_edges.iter().enumerate() {
        if index == 0 || index == target_edges.len() - 1 {
            continue;
        }

        let fiber = generate_new_fiber(network, edge, XCType::Fxc, XCType::Fxc);
        let xc = network.get_xc_mut_on_node(edge.src.into(), &XCType::Fxc);
        xc.connect_io(&prev_dst_device_id[0], &fiber.src_port_ids[0])
            .unwrap();

        prev_dst_device_id = fiber.dst_port_ids.clone();
        edge_type_tuples.push((*edge, XCType::Fxc, XCType::Fxc));
        fibers.push(fiber);
    }

    // 最後のファイバ (FXC → WXC)
    let last_edge = target_edges.last().unwrap();
    let last_fiber = generate_new_fiber(network, last_edge, XCType::Fxc, XCType::Wxc);
    let xc = network.get_xc_mut_on_node(last_edge.src.into(), &XCType::Fxc);
    xc.connect_io(&prev_dst_device_id[0], &last_fiber.src_port_ids[0])
        .unwrap();

    edge_type_tuples.push((*last_edge, XCType::Fxc, XCType::Wxc));
    fibers.push(last_fiber);

    // デバッグログ
    debugger::log_fibers_expand(config, network, &fibers);
    debugger::log_fxc_bypass(config, target_edges);

    // 登録
    network.regist_fibers(fibers.clone());

    (fibers, edge_type_tuples)
}

/// `sub` が `sup` に連続して含まれているかどうか
fn is_subsequence(sub: &[Edge], sup: &[Edge]) -> bool {
    if sub.len() > sup.len() {
        return false;
    }

    sup.windows(sub.len()).any(|window| window == sub)
}

/// FXCバイパスを新設する
pub fn expand_fxc_fibers(config: &Config, network: &mut Network, target_edges: &[Edge]) {
    if target_edges.len() < 2 {
        panic!(
            "target_edges: {:?} should be longer than two.",
            target_edges
        );
    }

    // ファイバ集合
    let mut fibers: Vec<Fiber> = vec![];

    // 最初のファイバを作成
    let first_fiber = generate_new_fiber(
        network,
        target_edges.first().unwrap(),
        XCType::Wxc,
        XCType::Fxc,
    );
    let mut prev_dst_device_id = first_fiber.dst_port_ids.clone();

    fibers.push(first_fiber);

    // 中間のファイバを作成
    for (index, edge) in target_edges.iter().enumerate() {
        if index == 0 || index == target_edges.len() - 1 {
            continue;
        }

        let imediate_fiber = generate_new_fiber(network, edge, XCType::Fxc, XCType::Fxc);

        // Connect to before fiber (new)
        let xc = network.get_xc_mut_on_node(edge.src.into(), &XCType::Fxc);
        xc.connect_io(&prev_dst_device_id[0], &imediate_fiber.src_port_ids[0])
            .unwrap();

        prev_dst_device_id = imediate_fiber.dst_port_ids.clone();

        fibers.push(imediate_fiber);
    }

    // 最後のファイバを作成
    let last_fiber = generate_new_fiber(
        network,
        target_edges.last().unwrap(),
        XCType::Fxc,
        XCType::Wxc,
    );

    // Connect to before fiber (new)
    let xc = network.get_xc_mut_on_node(target_edges.last().unwrap().src.into(), &XCType::Fxc);
    match xc.connect_io(&prev_dst_device_id[0], &last_fiber.src_port_ids[0]) {
        Ok(_) => (),
        Err(_err) => {
            println!("{:?}", target_edges);
            panic!()
        }
    }

    fibers.push(last_fiber);

    // debug
    debugger::log_fibers_expand(config, network, &fibers);
    debugger::log_fxc_bypass(config, target_edges);

    network.regist_fibers(fibers);
}

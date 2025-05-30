use crate::{
    config::Config,
    debugger,
    network::{Fiber, Network, XCType},
    Edge,
};

use super::generate_new_fiber;

/// FXCバイパスを新設する
pub fn expand_fxc_fibers(
    config: &Config,
    network: &mut Network,
    target_edges: &[Edge],
    all_installed_edges: &[Vec<Edge>],
) -> (Vec<Fiber>, Vec<(Edge, XCType, XCType)>) {
    if target_edges.len() < 2 {
        panic!(
            "target_edges: {:?} should be longer than two.",
            target_edges
        );
    }

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

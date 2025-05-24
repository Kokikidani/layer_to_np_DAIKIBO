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
) {

    if target_edges.len() < 2 {
        panic!("target_edges: {:?} should be longer than two.", target_edges);
    }

    // ファイバ集合
    let mut fibers: Vec<Fiber> = vec![];

    // 最初のファイバを作成
    let first_fiber = generate_new_fiber(
        network,
        target_edges.first().unwrap(),
        XCType::Wxc,
        XCType::Fxc
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
        xc.connect_io(&prev_dst_device_id[0], &imediate_fiber.src_port_ids[0]).unwrap();

        prev_dst_device_id = imediate_fiber.dst_port_ids.clone();

        fibers.push(imediate_fiber);
    }

    // 最後のファイバを作成
    let last_fiber = generate_new_fiber(
        network,
        target_edges.last().unwrap(),
        XCType::Fxc,
        XCType::Wxc
    );

    // Connect to before fiber (new)
    let xc = network.get_xc_mut_on_node(target_edges.last().unwrap().src.into(), &XCType::Fxc);
    match xc.connect_io(&prev_dst_device_id[0], &last_fiber.src_port_ids[0]) {
        Ok(_) => (),
        Err(_err) => {
            println!("{:?}", target_edges);
            panic!()
        },
    }

    fibers.push(last_fiber);

    // debug
    debugger::log_fibers_expand(config, network, &fibers);
    debugger::log_fxc_bypass(config, target_edges);

    network.regist_fibers(fibers);
}

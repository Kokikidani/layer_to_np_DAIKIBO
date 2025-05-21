use indicatif::{ ProgressBar, ProgressStyle };
use rand::Rng;

use crate::{
    config::Config,
    debugger,
    demand::Demand,
    network::{Fiber, FiberID, Network},
    np_core::parameters::{ PB_CHARS, PB_TEMPLATES },
    topology::{get_random_shortest_path, Topology},
};

use super::{expander, get_expand_edges, pathfinder };

pub fn delete_all_paths(network: &mut Network, demand_list: &mut [Demand]) {
    for demand in demand_list {
        delete(demand, network);
    }
}

pub fn assign_all_paths(
    config: &Config,
    network: &mut Network,
    topology: &Topology,
    demand_list: &mut [Demand]
) {
    let pb = ProgressBar::new(demand_list.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar().template(PB_TEMPLATES).unwrap().progress_chars(PB_CHARS)
    );

    // 一旦，パスをWXCネットワークに収容，足りない部分は増設
    let mut i = 0;
    while i < demand_list.len() {
        let demand = &mut demand_list[i];

        if assign(config, demand, topology, network) {
            // debug
            debugger::log_demand_assign(config, network, demand);
            pb.inc(1);
            i += 1;
        } else {
            // 使用していないファイバを削除
            // network.delete_empty_fibers(config);

            // debug
            // debugger::log_state_matrix(config, network);

            // 最短経路上にあるWXCファイバを選択，
            // let shortest_route_cand = &topology.route_candidates.get(&demand.sd).unwrap()[0];
            let shortest_route_cand = get_random_shortest_path(topology, &demand.sd, network.rng.gen_range(0..u64::MAX), None);

            // 拡張すべきエッジを取得
            let expand_edges = get_expand_edges(network, &shortest_route_cand.node_route);

            // 拡張
            // expander::expand_wxc_fibers_with_edge(config, network, &expand_edges);
            expander::expand_wxc_fibers(config, network, &expand_edges);
        }

        debugger::log_state_matrix(config, network);
    }
    pb.finish_and_clear();
}

pub fn assign(
    config: &Config,
    demand: &mut Demand,
    topology: &Topology,
    network: &mut Network
) -> bool {
    if let Some(assignment_instruction) = pathfinder::search(config, demand, topology, network) {
        network.assign_path(assignment_instruction.slot_head.clone(), &assignment_instruction.fiber_ids, &assignment_instruction.core_indices, demand);

        // Demandへ情報を適用
        demand.slot_heads = assignment_instruction.slot_head;
        demand.fiber_ids = assignment_instruction.fiber_ids;
        demand.slot_width = assignment_instruction.slot_width;
        demand.core_indices = assignment_instruction.core_indices;

        true
    } else {
        false
    }
}

fn delete(demand: &mut Demand, network: &mut Network) {
    if !demand.slot_heads.is_empty() {
        network.remove_path(demand);
        demand.reset();
    }
}

/// 最も使用率の低いファイバを取得して返す関数
pub fn get_least_utilized_fiber<'a>(
    network: &'a Network,
) -> Option<(&'a FiberID, &'a Fiber, f64)> {
    let mut min_utilization_fiber: Option<(&FiberID, &Fiber, f64)> = None;

    for (fiber_id, fiber) in network.get_all_fibers().iter() {
        let used_slots = fiber.count_used_slots();
        let total_slots = fiber.total_slots();

        if total_slots == 0 {
            continue; // 0除算を避ける
        }

        let utilization = used_slots as f64 / total_slots as f64;

        match min_utilization_fiber {
            Some((_, _, min_util)) if utilization >= min_util => {}
            _ => {
                min_utilization_fiber = Some((fiber_id, fiber, utilization));
            }
        }
    }

    min_utilization_fiber
}

pub fn peek_least_utilized_fiber<'a>(network: &'a Network) -> Option<&'a Fiber> {
    if let Some((_fiber_id, fiber, utilization)) = get_least_utilized_fiber(network) {
        println!(
            "Least utilized fiber → Edge: {:?}, Utilization: {:.2}%",
            fiber.edge,
            utilization * 100.0
        );
        Some(fiber)
    } else {
        println!("⚠ No removable fiber found.");
        None
    }
}

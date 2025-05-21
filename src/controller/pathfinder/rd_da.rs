use fxhash::FxHashMap;

use crate::{
    demand::Demand,
    network::{CoreIndex, FiberID, Network},
    np_core::StateMatrix,
    topology::{ RouteCandidate, Topology },
    Edge,
};

use super::{assignemnt_instruction::AssignmentInstruction, get_empty_fiber_core_routes};

fn calc_route_cand_costs(
    edges_cost: &FxHashMap<Edge, f64>,
    route_cands: &[RouteCandidate]
) -> Vec<f64> {
    let mut edge_routes_cost: Vec<f64> = Vec::with_capacity(route_cands.len());
    for route_cand in route_cands {
        let edge_route = &route_cand.edge_route;

        let mut cost: f64 = 0.0;
        for &edge in edge_route {
            cost += edges_cost.get(&edge).unwrap_or(&0.0);
        }

        edge_routes_cost.push(cost);
    }
    edge_routes_cost
}

/// Search function considering Distance Adaptive Modulation
pub fn search(
    demand: &Demand,
    topology: &Topology,
    network: &mut Network
) -> Option<AssignmentInstruction> {
    // Get route candidate of the demand SD
    let route_cands: &Vec<RouteCandidate> = topology.route_candidates.get(&demand.sd).unwrap();

    // Get the cost of all edge_routes
    let edge_routes_cost: Vec<f64> = calc_route_cand_costs(network.get_edge_cost(), route_cands);

    // Sort indices by the costs
    let mut route_cands_index_ordered: Vec<(usize, &f64)> = edge_routes_cost
        .iter()
        .enumerate()
        .collect(); // the cost of route_cand with index
    route_cands_index_ordered.sort_by(|(_a_index, a_cost), (_b_index, b_cost)|
        a_cost.partial_cmp(b_cost).unwrap()
    );

    // loop route_cands with the cost order
    for (index, _cost) in route_cands_index_ordered {
        // target route_cand of this loop
        let route_cand: &RouteCandidate = &route_cands[index];

        // Fiber route cand, selected with single slot search
        let fiber_core_route_cands: Vec<(Vec<FiberID>, Vec<CoreIndex>)> = get_empty_fiber_core_routes(network, route_cand, 1);

        for (fiber_route, core_indices) in &fiber_core_route_cands {
            let width = get_width(network, fiber_route);

            let mut target_state_matrix = StateMatrix::new();
            let mut flag = true;

            for (fiber_id, core_index) in fiber_route.iter().zip(core_indices.iter()) {
                let state_matrix_of_fiber = network.get_fiber_by_id(fiber_id).state_matrixes[core_index.index()];
                target_state_matrix |= state_matrix_of_fiber;

                if !target_state_matrix.has_empty_contiguous_slots(width) {
                    flag = false;
                    break;
                }
            }

            if flag {
                let slot = target_state_matrix.get_empty_contiguous_slots(width).unwrap();
                return Some({
                    AssignmentInstruction {
                        fiber_ids: fiber_route.clone(),
                        slot_head: vec![slot; fiber_route.len()],
                        slot_width: width,
                        core_indices: vec![CoreIndex::new(0); fiber_route.len()],
                    }
                });
            }
        }
    }

    None
}

/// 占有スロット数を計算
fn get_width(network: &Network, fiber_route: &[FiberID]) -> usize {
    let mut _quality_distance = 0;

    for fiber_id in fiber_route {
        _quality_distance += network.get_fiber_quality_distance_by_id(fiber_id);
    }

    // From quality distance to size
    unimplemented!();
    // 1
}

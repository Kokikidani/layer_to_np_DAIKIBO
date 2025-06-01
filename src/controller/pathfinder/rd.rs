use fxhash::FxHashMap;

use crate::{
    demand::Demand,
    network::{Network},
    topology::{RouteCandidate, Topology},
    Edge,
};

use super::{assignemnt_instruction::AssignmentInstruction, get_result_from_route_cand};

pub fn search(
    demand: &Demand,
    topology: &Topology,
    network: &mut Network,
) -> Option<AssignmentInstruction> {
    let route_cands = topology.route_candidates.get(&demand.sd).unwrap();

    let edge_routes_cost: Vec<f64> = calc_route_cand_costs(network.get_edge_cost(), route_cands);

    // インデックスをソート
    let mut route_cands_index_ordered: Vec<(usize, &f64)> =
        edge_routes_cost.iter().enumerate().collect();
    route_cands_index_ordered.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());

    for (index, _) in route_cands_index_ordered {
        let route_cand: &RouteCandidate = &route_cands[index];

        match get_result_from_route_cand(network, route_cand) {
            Some(result) => return Some(result),
            None => continue,
        }
    }
    None
}

fn calc_route_cand_costs(
    edges_cost: &FxHashMap<Edge, f64>,
    route_cands: &[RouteCandidate],
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

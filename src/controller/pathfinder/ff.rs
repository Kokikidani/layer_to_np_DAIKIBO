use crate::{ demand::Demand, network::Network, topology::Topology };

use super::{assignemnt_instruction::AssignmentInstruction, get_result_from_route_cand};

pub fn search(
    demand: &Demand,
    topology: &Topology,
    network: &mut Network
) -> Option<AssignmentInstruction> {
    let route_cands = topology.route_candidates.get(&demand.sd).unwrap();

    for route_cand in route_cands {
        match get_result_from_route_cand(network, route_cand) {
            Some(result) => return Some(result),
            None => continue,
        }
    }

    None
}

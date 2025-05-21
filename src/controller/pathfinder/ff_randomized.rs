use rand::Rng;

use crate::{
    demand::Demand,
    network::Network,
    topology::{ RouteCandidate, Topology },
    utils::shuffle_array,
};

use super::{assignemnt_instruction::AssignmentInstruction, get_result_from_route_cand};

pub fn search(
    demand: &Demand,
    topology: &Topology,
    network: &mut Network
) -> Option<AssignmentInstruction> {
    let rand_seed = network.rng.gen_range(0..u64::MAX);

    let route_cands = topology.route_candidates.get(&demand.sd).unwrap();

    let max_path_len = route_cands
        .iter()
        .max_by_key(|x| x.edge_route.len())
        .unwrap()
        .edge_route.len();
    let mut path_len = 0;

    loop {
        path_len += 1;

        let mut route_cands_slices: Vec<&RouteCandidate> = route_cands
            .iter()
            .filter(|p| p.edge_route.len() == path_len)
            .collect();
        shuffle_array(&mut route_cands_slices, rand_seed);

        for route_cand in route_cands_slices {
            match get_result_from_route_cand(network, route_cand) {
                Some(result) => return Some(result),
                None => continue,
            }
        }

        if path_len >= max_path_len {
            break;
        }
    }

    None
}

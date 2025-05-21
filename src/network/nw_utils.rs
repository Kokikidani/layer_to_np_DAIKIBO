use fxhash::FxHashMap;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::{config::Config, controller::expander::{expand_fxc_fibers, expand_sxc_fibers, expand_wbxc_fibers, expand_wxc_fibers}, network::FiberID, topology::Topology, Edge};

use super::{EdgesType, Network, XCType, XC};

pub fn network_from_hashmap(
    config: &Config,
    topology: &Topology,
    hashmap: FxHashMap<(EdgesType, Vec<Edge>), usize>
) -> Network {
    
    // Network::newとは初期ファイバ数の違いがある

    let fibers = FxHashMap::default();
    let fiber_id_on_edges: FxHashMap<Edge, Vec<FiberID>> = FxHashMap::default();

    let edges = topology.edges.clone();

    let xcs = FxHashMap::default();

    let edge_costs = FxHashMap::default();

    let mut empty_fiber_ids_on_edges_cache = FxHashMap::default();
    for edge in &edges {
        empty_fiber_ids_on_edges_cache.insert(*edge, vec![]);
    }
    let portid_to_xcid = FxHashMap::default();

    let rng = ChaCha8Rng::seed_from_u64(config.simulation.random_seed);
    let layer_topologies = FxHashMap::default();

    let mut network = Network {
        fibers,
        fiber_ids_on_edges: fiber_id_on_edges,
        edges,
        xcs,
        edge_costs,
        empty_fiber_ids_on_edges_cache,
        rng,
        portid_to_xcid,
        layer_topologies,
        // original_wxc2wxc_fiber_count: 0,
    };

    // for ((edges_type, edge_seq), count) in &hashmap {
    //     for _ in 0..*count {
    //         match edges_type {
    //             EdgesType::Wxc => expand_wxc_fibers(config, &mut network, edge_seq),
    //             EdgesType::Fxc => expand_fxc_fibers(config, &mut network, edge_seq),
    //             EdgesType::Wbxc => expand_wbxc_fibers(config, &mut network, edge_seq),
    //             EdgesType::Sxc => expand_sxc_fibers(config, &mut network, edge_seq),
    //         }
    //     }
    // }

    network
}

pub fn wxc_network_from_hashmap(
    config: &Config,
    topology: &Topology,
    hashmap: FxHashMap<(EdgesType, Vec<Edge>), usize>
) -> Network {
    let fibers = FxHashMap::default();
    let fiber_id_on_edges: FxHashMap<Edge, Vec<FiberID>> = FxHashMap::default();

    let edges = topology.edges.clone();

    let mut xcs = FxHashMap::default();
    for node in 0..topology.link_matrix.len() {
        let wxc = XC::new(node, XCType::Wxc);
        xcs.insert(wxc.id, wxc);
    }

    let edge_costs = FxHashMap::default();

    let mut empty_fiber_ids_on_edges_cache = FxHashMap::default();
    for edge in &edges {
        empty_fiber_ids_on_edges_cache.insert(*edge, vec![]);
    }

    let rng = ChaCha8Rng::seed_from_u64(config.simulation.random_seed);
    let portid_to_xcid = FxHashMap::default();
    let layer_topologies = FxHashMap::default();

    let mut network = Network {
        fibers,
        fiber_ids_on_edges: fiber_id_on_edges,
        edges,
        xcs,
        edge_costs,
        empty_fiber_ids_on_edges_cache,
        rng,
        portid_to_xcid,
        layer_topologies,
    };

    for (edge_seq, count) in &hashmap {
        for _ in 0..*count {
            expand_wxc_fibers(config, &mut network, &edge_seq.1);
        }
    }

    network
}



/// (総ファイバ数, WXC2WXCファイバ割合)
pub fn calc_fiber_proportions(fiber_breakdown: FxHashMap<[XCType; 2], usize>) -> (usize, f64) {
    let sum = fiber_breakdown.values().sum();
    let w2w_fibers_count = *fiber_breakdown.get(&[XCType::Wxc, XCType::Wxc]).unwrap_or(&0);
    let prop = w2w_fibers_count as f64 / sum as f64;

    (sum, prop)
}

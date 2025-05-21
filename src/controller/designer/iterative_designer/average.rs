//! Iterative design and average fiber counts.

// use crate::Edge;
// use fxhash::FxHashMap;
// use rand::SeedableRng;
// use rand_chacha::ChaCha8Rng;

use crate::{
    config::Config,
    // controller::designer::iterative_designer::get_results,
    network::Network,
    topology::Topology,
};

pub fn main(_config: &Config) -> (Network, Topology, String) {
    // let mut rng = ChaCha8Rng::seed_from_u64(config.simulation.random_seed);
    // let results = get_results(&mut rng, 10, config);

    // let mut tmp_edge_counter: FxHashMap<Vec<Edge>, usize> = FxHashMap::default();
    // for (network, _topology, _) in results.iter() {
    //     let advanced_edges = network.get_edges_advanced();
    //     for advanced_edge in advanced_edges {
    //         let entry = tmp_edge_counter.entry(advanced_edge).or_insert(0);
    //         *entry += 1;
    //     }
    // }

    // let mut edge_counter: FxHashMap<Vec<Edge>, usize> = FxHashMap::default();
    // for (tmp_edge, tmp_edge_count) in tmp_edge_counter {
    //     edge_counter.insert(
    //         tmp_edge,
    //         (tmp_edge_count as f64 / results.len() as f64).ceil() as usize,
    //     );
    // }

    // let topology = Topology::new(config);
    // let network = network::network_from_hashmap(config, &topology, edge_counter);

    // (network, topology, results[0].2.clone())

    panic!("This function will not work")
}

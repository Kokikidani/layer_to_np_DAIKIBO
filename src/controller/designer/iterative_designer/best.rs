use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::{
    config::Config,
    controller::{ designer::iterative_designer::get_results, output::{self} },
    network::{ self, Network, XCType },
    np_core::parameters::MEAN_N,
    topology::Topology,
};

pub fn main(config: &Config, xc_types: &[XCType; 2]) -> (Network, Topology, String) {

    output::save_config(config, &config.simulation.outdir);

    let mut rng = ChaCha8Rng::seed_from_u64(config.simulation.random_seed);
    let results = get_results(&mut rng, MEAN_N, config, xc_types);

    let mut best_score = (0, 0.0);
    for (index, (network, _topology, _)) in results.iter().enumerate() {
        let fiber_breakdown = network.get_fiber_breakdown();
        let sum: usize = fiber_breakdown.values().sum();
        let fxc_related = sum - fiber_breakdown.get(&[XCType::Wxc, XCType::Wxc]).unwrap_or(&0);
        let score = (fxc_related as f64) / (sum as f64);
        // similar function exists

        if best_score.1 < score {
            best_score = (index, score);
        }
    }

    let best_outdir = results[best_score.0].2.clone();
    output::save_best(&config.simulation.outdir, &format!("{}", best_score.0));

    let topology = Topology::new(config);
    let network = network::network_from_hashmap(
        config,
        &topology,
        results[best_score.0].0.export()
    );

    (network, topology, best_outdir)
}

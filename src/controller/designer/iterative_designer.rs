use rand_chacha::ChaCha8Rng;

use crate::{
    config::Config, controller::designer::main, network::{Network, XCType}, topology::Topology
};
use rand::Rng;

mod average;
mod best;

pub use average::main as average_main;
pub use best::main as best_main;

pub fn get_results(
    rng: &mut ChaCha8Rng,
    n: usize,
    config: &Config,
    xc_types: &[XCType; 2]
) -> Vec<(Network, Topology, String)> {
    let mut results = vec![];

    for i in 0..n {
        let new_seed = rng.gen_range(0..i64::MAX as u64);
        println!("\n\nNEW SEED: {}", new_seed);

        let mut new_config = config.clone();
        new_config.simulation.random_seed = new_seed;
        new_config.simulation.outdir = format!(
            "{}/{:02}_{:010}",
            config.simulation.outdir,
            i,
            new_seed
        );

        let (network, topology, output_dir) = main(&new_config, xc_types);

        results.push((network, topology, output_dir));
    }

    results
}

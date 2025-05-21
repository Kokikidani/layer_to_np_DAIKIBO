use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::{ config::Config, np_core::dist::get_poisson_interval, topology::Topology };

use super::{
    find_min_position_in_2d_matrix,
    get_traffic_matrix,
    get_uniform_traffic_matrix,
    normalize_traffic_distribution_matrix,
    Demand,
    LAMBDA_C,
    SD,
};

pub fn get_dynamic_demand_list(
    config: &Config,
    topology: &Topology,
    traffic_intensity: f64
) -> Vec<Demand> {
    let traffic_filename = match config.traffic.distribution_filepath.as_str() {
        "" => None,
        _ => {
            let name = config.traffic.distribution_filepath.as_str();

            if !name.contains(&config.network.topology) {
                eprintln!(
                    "[WARNING] Traffic Distribution Filepath doesn't contain topology name. Is this correct?"
                );
            }

            Some(name)
        }
    };

    {
        let seed = config.simulation.random_seed;
        let path_num = config.traffic.path_num;
        let node_count = topology.link_matrix.len();
        if traffic_intensity <= 0.0 {
            eprintln!("`traffic_intensity` needs to be large than 0.0; Demand isn't generated");
            return vec![];
        }

        let mut traffic_distribution_matrix = match traffic_filename {
            Some(name) => get_traffic_matrix(name),
            None => get_uniform_traffic_matrix(node_count),
        };
        normalize_traffic_distribution_matrix(&mut traffic_distribution_matrix, traffic_intensity);

        let mut traffic_arrival_time_table = vec![vec![0; node_count]; node_count];
        for (i, row) in traffic_arrival_time_table.iter_mut().enumerate() {
            for (j, value) in row.iter_mut().enumerate() {
                if traffic_distribution_matrix[i][j] == 0.0 {
                    *value = usize::MAX;
                }
            }
        }

        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        let mut demand_list = Vec::with_capacity(path_num);
        for i in 0..path_num {
            let (src, dst) = find_min_position_in_2d_matrix(&traffic_arrival_time_table);
            let sd_edge = SD::new(src, dst);

            let start = traffic_arrival_time_table[src][dst];

            let traffic_intensity = traffic_distribution_matrix[src][dst];
            let duration = get_poisson_interval(&mut rng, LAMBDA_C);

            let demand = Demand::new(sd_edge, i, start, duration);
            demand_list.push(demand);

            let lambda = traffic_intensity * LAMBDA_C;
            let interval = get_poisson_interval(&mut rng, lambda);

            traffic_arrival_time_table[src][dst] += interval;
        }

        demand_list
    }
}

#[test]
fn demand_new_test() {
    let sd = SD::new(0, 1);
    let demand = Demand::new(sd, 0, 1, 2);
    assert_eq!(demand.end_time, 3);
    println!("{:?}", demand);
}

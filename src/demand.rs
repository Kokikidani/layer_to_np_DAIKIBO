use std::{ fs::File, io::Read };

use crate::{
    config::Config, network::{CoreIndex, FiberID}, np_core::dist::get_poisson_interval, topology::{ get_ave_shortest_hops, Topology }, SD, SLOT
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

pub mod dynamic;

const LAMBDA_C: f64 = 1.0 / 3000.0;

#[derive(Debug, Clone)]
pub struct Demand {
    pub sd: SD,
    pub fiber_ids: Vec<FiberID>,
    pub slot_heads: Vec<usize>,
    pub slot_width: usize,
    pub core_indices: Vec<CoreIndex>,
    pub index: usize,
    pub start_time: usize,
    pub end_time: usize,
    pub duration: usize,
    pub data_speed: usize, // Gbps
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ModurationFromat {
    Qpsk = 0,
    Qam8 = 1,
    Qam16 = 2,
    Qam32 = 3,
}

impl Demand {
    pub fn reset(&mut self) {
        self.fiber_ids.clear();
        self.slot_heads.clear();
        self.slot_width = 0;
        self.core_indices.clear();
    }

    pub fn new(sd: SD, index: usize, start: usize, duration: usize) -> Self {
        let end = start + duration;
        Self {
            sd,
            fiber_ids: vec![],
            slot_heads: vec![],
            index,
            start_time: start,
            end_time: end,
            duration,
            data_speed: 0,
            slot_width: 0,
            core_indices: vec![],
        }
    }
}

pub fn get_demand_list(config: &Config, topology: &Topology) -> Vec<Demand> {
    let traffic_intensity = config.simulation.traffic_intensity;

    // Traffic intensity checking
    {
        let num_links = topology.edges.len();
        let num_nodes = topology.link_matrix.len();
        let ave_shortest_hops = get_ave_shortest_hops(topology);
        let normalized_traffic_intensity =
            ((num_links * SLOT) as f64) /
            ((num_nodes as f64) * ((num_nodes - 1) as f64) * ave_shortest_hops);

        if normalized_traffic_intensity > traffic_intensity {
            eprintln!(
                "[WARNING] Traffic Intensity: {:.2} is too low to fulfill fibers on links when a link has one fiber.",
                traffic_intensity
            );
            eprintln!(
                "[  INFO ] Normalized Traffic Intensity is {:.2}",
                normalized_traffic_intensity
            );
        }
    }
    // Traffic intensity checking end

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

    let lib_demand_list = get_static_demand_list(
        config.simulation.random_seed,
        topology.link_matrix.len(),
        config.simulation.traffic_intensity,
        traffic_filename
    );

    lib_demand_list
        .into_iter()
        .enumerate()
        .map(|(index, x)| Demand::new(x.sd, index, 0, usize::MAX))
        .collect()
}

/// CSV形式の二次元配列を読み込む
pub fn string_to_vec2_f64(data: &str) -> Vec<Vec<f64>> {
    data.trim()
        .lines()
        .map(|line| {
            line.trim()
                .split(',')
                .filter_map(|v| v.trim().parse().ok())
                .collect()
        })
        .collect()
}

fn get_traffic_matrix(name: &str) -> Vec<Vec<f64>> {
    let filename = format!("./files/population/{}.csv", name);

    match File::open(filename) {
        Ok(mut file) => {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                string_to_vec2_f64(&content)
            } else {
                panic!("ファイルを読み込めせんでした")
            }
        }
        Err(_) => panic!("ファイルを開けませんでした"),
    }
}

fn get_uniform_traffic_matrix(node_count: usize) -> Vec<Vec<f64>> {
    let traffic_intensity = 1.0; // Traffic intensity should be changed out of this function.

    // Uniform distribution matrix creation
    let mut traffic_distribution_matrix = vec![vec![traffic_intensity; node_count]; node_count];
    for (i, item) in traffic_distribution_matrix.iter_mut().enumerate().take(node_count) {
        item[i] = 0.0;
    }
    traffic_distribution_matrix
}

/// Normalize 2d f64 matrix. to `row * (row - 1)`.
/// This function is not needed for static traffic distribution, but used for dynamic traffic distribution in the future.
fn normalize_traffic_distribution_matrix(matrix: &mut [Vec<f64>], traffic_intensity: f64) {
    let sum: f64 = matrix
        .iter()
        .map(|r| r.iter().sum::<f64>())
        .sum();
    let node_count = matrix.len() as f64;
    let target = traffic_intensity * node_count * (node_count - 1.0);
    let scale = target / sum;

    for row in matrix {
        for v in row {
            *v *= scale;
        }
    }
}

/// Find minimum number and returns index of 2d matrix.
/// Return format is tuple of (row, col).
fn find_min_position_in_2d_matrix(matrix: &[Vec<usize>]) -> (usize, usize) {
    let mut min_value = matrix[0][0];
    let mut min_index = (0, 0);

    for (i, row) in matrix.iter().enumerate() {
        for (j, &value) in row.iter().enumerate() {
            if value < min_value {
                min_value = value;
                min_index = (i, j);
            }
        }
    }

    min_index
}

pub fn get_static_demand_list(
    seed: u64,
    node_count: usize,
    traffic_intensity: f64,
    traffic_filename: Option<&str>
) -> Vec<Demand> {
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
    let path_num = (traffic_intensity * (node_count as f64) * ((node_count - 1) as f64)) as usize;
    let mut demand_list = Vec::with_capacity(path_num);

    for index in 0..path_num {
        let (src, dst) = find_min_position_in_2d_matrix(&traffic_arrival_time_table);
        let sd_edge = SD::new(src, dst);
        let start = traffic_arrival_time_table[src][dst];
        let traffic_intensity = traffic_distribution_matrix[src][dst];
        let duration = get_poisson_interval(&mut rng, LAMBDA_C);

        let demand = Demand::new(sd_edge, index, start, duration);
        demand_list.push(demand);

        let lambda = traffic_intensity * LAMBDA_C;
        let interval = get_poisson_interval(&mut rng, lambda);

        traffic_arrival_time_table[src][dst] += interval;
    }

    demand_list
}

// pub fn get_static_demand_list(
//     seed: u64,
//     node_count: usize,
//     traffic_intensity: f64,
//     traffic_filename: Option<&str>
// ) -> Vec<Demand> {
//     let mut traffic_distribution_matrix = match traffic_filename {
//         Some(name) => get_traffic_matrix(name),
//         None => get_uniform_traffic_matrix(node_count),
//     };
//     normalize_traffic_distribution_matrix(&mut traffic_distribution_matrix, traffic_intensity);

//     let mut traffic_arrival_time_table = vec![vec![0; node_count]; node_count];
//     for (i, row) in traffic_arrival_time_table.iter_mut().enumerate() {
//         for (j, value) in row.iter_mut().enumerate() {
//             if traffic_distribution_matrix[i][j] == 0.0 {
//                 *value = usize::MAX;
//             }
//         }
//     }

//     let mut rng = ChaCha8Rng::seed_from_u64(seed);
//     let path_num = (traffic_intensity * (node_count as f64) * ((node_count - 1) as f64)) as usize;

//     // 光パスの最大数を6000に制限
//     let max_path_num = 6000;
//     let limited_path_num = path_num.min(max_path_num);

//     let mut demand_list = Vec::with_capacity(limited_path_num);

//     for index in 0..limited_path_num {
//         let (src, dst) = find_min_position_in_2d_matrix(&traffic_arrival_time_table);
//         let sd_edge = SD::new(src, dst);
//         let start = traffic_arrival_time_table[src][dst];
//         let traffic_intensity = traffic_distribution_matrix[src][dst];
//         let duration = get_poisson_interval(&mut rng, LAMBDA_C);

//         let demand = Demand::new(sd_edge, index, start, duration);
//         demand_list.push(demand);

//         let lambda = traffic_intensity * LAMBDA_C;
//         let interval = get_poisson_interval(&mut rng, lambda);

//         traffic_arrival_time_table[src][dst] += interval;
//     }

//     demand_list
// }

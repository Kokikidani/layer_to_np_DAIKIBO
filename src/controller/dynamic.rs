use crate::demand::Demand;
use crate::np_core::parameters::{ PB_CHARS, PB_TEMPLATES };
use crate::{
    config::Config,
    debugger,
    demand::dynamic::get_dynamic_demand_list,
    network::Network,
    topology::Topology,
    THREADS,
};
use indicatif::MultiProgress;
use indicatif::{ ProgressBar, ProgressStyle };
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

use super::ctrl_utils::assign;

pub fn get_blocking_curve(
    config: &Config,
    network: &Network,
    topology: &Topology,
    traffic_intensity: &[f64]
) -> Vec<(f64, f64)> {
    let pool = ThreadPoolBuilder::new()
        .num_threads(THREADS)
        .build()
        .expect("Failed to create thread pool");

    let m = MultiProgress::new();

    let tis_pbs: Vec<(f64, ProgressBar)> = traffic_intensity.iter().map(|ti| (*ti, m.add(ProgressBar::new(config.traffic.path_num as u64)))).collect();

    pool.install(|| {
        tis_pbs
            .into_par_iter()
            .map(|(ti, pb)| {
                let mut tmp_network = network.clone();
                let blocking_rate = dynamic_analysis(
                    config,
                    &mut tmp_network,
                    topology,
                    ti,
                    Some(pb)
                );
                // println!("{} {}", ti, blocking_rate);
                (ti, blocking_rate)
            })
            .collect()
    })
}

pub fn dynamic_analysis(
    config: &Config,
    network: &mut Network,
    topology: &Topology,
    traffic_intensity: f64,
    progressbar: Option<ProgressBar>
) -> f64 {
    // パス需要
    let mut demand_list = get_dynamic_demand_list(config, topology, traffic_intensity);

    let mut assigned_demand_indices = vec![];

    let mut block_count = 0;
    let pb = match progressbar {
        Some(pb) => pb,
        None => ProgressBar::new(demand_list.len() as u64),
    };

    pb.set_style(
        ProgressStyle::default_bar().template(PB_TEMPLATES).unwrap().progress_chars(PB_CHARS)
    );

    for i in 0..demand_list.len() {
        let current_time = demand_list[i].start_time;

        match assign(config, &mut demand_list[i], topology, network) {
            true => {
                assigned_demand_indices.push(i);
                debugger::log_demand_assign(config, network, &demand_list[i]);
            }
            false => {
                block_count += 1;
            }
        }

        dynamic_delete(network, &mut demand_list, current_time, &mut assigned_demand_indices);

        // debug
        debugger::log_state_matrix(config, network);

        pb.inc(1);
    }

    let blocking_rate = (block_count as f64) / (demand_list.len() as f64);
    pb.finish();
    // pb.finish_with_message(
    //     format!("{:.2} {:.5}", traffic_intensity, blocking_rate)
    // );

    blocking_rate
}

fn dynamic_delete(
    network: &mut Network,
    demand_list: &mut [Demand],
    current_time: usize,
    assigned_demand_indices: &mut Vec<usize>
) {
    let mut deleted_demand_indices = Vec::with_capacity(assigned_demand_indices.len());

    for demand_index in assigned_demand_indices.iter_mut() {
        let dynamic_demand = &mut demand_list[*demand_index];

        if dynamic_demand.end_time <= current_time {
            network.remove_path(dynamic_demand);
            dynamic_demand.reset();

            deleted_demand_indices.push(*demand_index);
        }
    }

    assigned_demand_indices.retain(|f| !deleted_demand_indices.contains(f));
}

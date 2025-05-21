//! Layer to NP 2
//!
//! レイヤ化異粒度ネットワークの評価を行うプログラム
//!
//! ## 目標
//!
//! OFC, Journal

mod config;
mod controller;
mod debugger;
mod demand;
mod network;
mod np_core;
mod topology;
mod utils;

use controller::{dynamic::get_blocking_curve, output::{self, save_blocking_curve}};
use network::{network_from_hashmap, wxc_network_from_hashmap};
pub use np_core::{ Edge, Node, SD, WBIndex };

use np_core::parameters::{ CURVE_RANGE_BOTTOM, CURVE_RANGE_UP, SLOT, THREADS };
use utils::arange;

use std::env;

fn main() {
    eprintln!("SLOT: {}\tTHREADS: {}", SLOT, THREADS);
    
    // Thead Checking
    let num_threads = num_cpus::get();
    if num_threads < THREADS {
        eprintln!("[WARNING] THREADS: {} is larger than num_cpus: {}.", THREADS, num_threads);
    }

    let args: Vec<String> = env::args().collect();
    let mut config = if args.len() == 2 {
        config::Config::new(&args[1])
    } else {
        config::Config::new("./config.toml")
    };
    output::init_master_dir(&mut config);

    let (network, _topology, _specific_outdir) = controller::main(&config);

    // {
    //     let tis: Vec<f64> = arange(
    //         config.simulation.traffic_intensity - CURVE_RANGE_BOTTOM,
    //         config.simulation.traffic_intensity + CURVE_RANGE_UP,
    //         0.1
    //     ).collect();

    //     let wxc_network = wxc_network_from_hashmap(&config, &_topology, network.export());
    //     let wxc_curve = get_blocking_curve(&config, &wxc_network, &_topology, &tis);
    //     let proposed_network = network_from_hashmap(&config, &_topology, network.export());
    //     let proposed_curve = get_blocking_curve(&config, &proposed_network, &_topology, &tis);

    //     save_blocking_curve(&config, &config.simulation.outdir, &wxc_curve, &proposed_curve);

    // }
    
}

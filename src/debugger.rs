use console::style;

use crate::{
    config::Config, controller::output, demand::Demand, network::{
        nw_utils::calc_fiber_proportions, state_matrix::{
            state_matrix_label_wo_w2w_from_network,
            state_matrix_wo_w2w_from_network,
        }, CoreIndex, Fiber, Network, XCType
    }, np_core::parameters::{MAX_BYPASS_LEN, MIN_BYPASS_LEN}, utils::enumerate_subsequences, Edge, WBIndex, SD
};

pub mod analysis;

pub fn log_alert(msg: &str) {
    println!("{:>8}| {}", style("ALERT").red(), style(msg).bold());
}

pub fn log_analysis(config: &Config, network: &Network, conv_nw_w2w_fiber_count: usize, demand_list: &[Demand]) {
    if config.debug.log_analysis {
        let fiber_breakdown = network.get_fiber_breakdown();
        let (sum, w2w_prop) = calc_fiber_proportions(fiber_breakdown);
        print!(
            "{:>8}| {:3} {:5.1}% {:.3} {:.3} {:.3}",
            style("ANALYSIS").blue(),
            sum, w2w_prop*100.0, 
            analysis::calc_fiber_count_ratio(network, conv_nw_w2w_fiber_count),
            analysis::calc_max_wxc_size(network),
            output::calc_wxc_pass_count_average(network, demand_list)
        );
        
        if config.debug.log_bypass {
            print!("\t");
        } else {
            println!();
        }
    }
}

pub fn log_net_analysis(config: &Config, network: &Network, conv_nw_w2w_fiber_count: usize, demand_list: &[Demand]) {
    if config.debug.log_analysis {
        let fiber_breakdown = network.get_fiber_breakdown();
        let (sum, w2w_prop) = calc_fiber_proportions(fiber_breakdown);
        println!(
            "{:>8}| {:3} {:5.1}% {:.3} {:.3} {:.3}",
            style("ANALYSIS").blue(),
            sum, w2w_prop*100.0, 
            analysis::calc_fiber_count_ratio(network, conv_nw_w2w_fiber_count),
            analysis::calc_max_wxc_size(network),
            output::calc_wxc_pass_count_average(network, demand_list)
        );
    }
}

pub fn log_demand_assign(config: &Config, network: &Network, demand: &Demand) {
    if config.debug.log_demand_assign {
        // edge_route, fiber_typeの取得
        let mut edge_route: Vec<Edge> = Vec::with_capacity(demand.fiber_ids.len());
        let mut xc_route = vec![network.get_fiber_sd_xc_type_by_id(demand.fiber_ids.first().unwrap())[0]];

        for fiber_id in &demand.fiber_ids {
            edge_route.push(network.get_fiber_by_id(fiber_id).edge);
            xc_route.push(network.get_fiber_sd_xc_type_by_id(fiber_id)[1]);
        }

        // node_routeの取得
        let node_route: Vec<usize> = {
            let target: &[Edge] = &edge_route;
            let mut output = vec![];

            if target.is_empty() {
                output
            } else {
                output.push(target[0].src.into());

                for tuple in target {
                    output.push(tuple.dst.into());
                }

                output
            }
        };

        println!("{:>8}|{:5} {:?} {:?} {:?}", style("ASSIGN").green(), demand.index, demand.slot_heads, node_route, xc_route);
    }
}

pub fn log_fibers_expand(config: &Config, network: &Network, fibers: &[Fiber]) {
    if config.debug.log_fiber_expand {
        for fiber in fibers {
            log_fiber_expand(config, network, fiber);
        }
    }
}

pub fn log_fiber_expand(config: &Config, network: &Network, fiber: &Fiber) {
    if config.debug.log_fiber_expand {
        let fiber_type = network.get_fiber_sd_xc_type(fiber);
        println!("{:>8}| {} => {} | {}", style("EXPAND").yellow(), fiber_type[0], fiber_type[1], fiber.edge);
    }
}

pub fn log_fiber_remove(config: &Config, fiber: &Fiber) {
    if config.debug.log_fiber_remove {
        let fiber_type = fiber.sd_xc_type;
        println!("{:>8}| {} => {} | {}", style("REMOVE").red(), fiber_type[0], fiber_type[1], fiber.edge);
    }
}

pub fn log_taboo_list_addition(config: &Config, sd: &SD) {
    if config.debug.log_taboo {
        println!("{:>8}| {}", style("TABOO").magenta(), sd);
    }
}

pub fn log_fxc_bypass(config: &Config, target_edges: &[Edge]) {
    if config.debug.log_bypass {
        print!("{:>8}| {}", style("BYPASS").yellow(), target_edges.first().unwrap());
        for edge in target_edges.iter().skip(1) {
            print!(" => {}", edge);
        }
        println!();
    }
}
// pub fn log_fxc_bypass_remove(config: &Config, target_edges: &[Edge]) {
//     if config.debug.log_bypass {
//         print!("{:>8}| {}", style("R_BYPASS").red(), target_edges.first().unwrap());
//         for edge in target_edges.iter().skip(1) {
//             print!(" => {}", edge);
//         }
//         println!();
//     }
// }

pub fn log_core_bypass(config: &Config, target_edges: &[Edge], core_index: &CoreIndex) {
    if config.debug.log_bypass {
        print!("{:>8}| {:?} | {}", style("BYPASS").yellow(), core_index, target_edges.first().unwrap());
        for edge in target_edges.iter().skip(1) {
            print!(" => {}", edge);
        }
        println!();
    }
}
pub fn log_core_bypass_remove(config: &Config, target_edges: &[Edge], core_index: &CoreIndex) {
    if config.debug.log_bypass {
        print!("{:>8}| {:?} | {}", style("R_BYPASS").red(), core_index, target_edges.first().unwrap());
        for edge in target_edges.iter().skip(1) {
            print!(" => {}", edge);
        }
        println!();
    }
}

pub fn log_wb_bypass(config: &Config, target_edges: &[Edge], wb_index: &WBIndex) {
    if config.debug.log_bypass {
        print!("{:>8}| {:?} | {}", style("BYPASS").yellow(), wb_index, target_edges.first().unwrap());
        // print!("{:>8}| {}", style("BYPASS").yellow(), target_edges.first().unwrap());
        for edge in target_edges.iter().skip(1) {
            print!(" => {}", edge);
        }
        println!();
    }
}
pub fn log_wb_bypass_remove(config: &Config, target_edges: &[Edge], wb_index: &WBIndex) {
    if config.debug.log_bypass {
        print!("{:>8}| {:?} | {}", style("R_BYPASS").red(), wb_index, target_edges.first().unwrap());
        for edge in target_edges.iter().skip(1) {
            print!(" => {}", edge);
        }
        println!();
    }
}

pub fn log_state_matrix(config: &Config, network: &Network) {
    // if config.network.expand_mode.as_str() != "wxc" {
    // log_state_matrix_wo_w2w(config, network);
    // } else {
    if config.debug.log_state_matrix {

        let mut fiber_count = 0;
        for edge in &network.edges {
            let fiber_ids_on_edge = network.get_fiber_id_on_edge(edge);
            for fiber_id in fiber_ids_on_edge {
                let fiber = network.get_fiber_by_id(&fiber_id);
                let [src_xc, dst_xc] = network.get_fiber_sd_xc_type(fiber);
                if !(src_xc == XCType::Wxc && dst_xc == XCType::Wxc) {
                    for (core_index, state_matrix) in fiber.state_matrixes.iter().enumerate() {
                        println!("{fiber_count:3} | {edge} | {src_xc}=>{dst_xc} | {core_index} | {state_matrix}");
                    }
                    fiber_count += 1;
                }
            }
        }
        // let label = state_matrix_label_from_network(network);
        // for (r_index, row) in state_matrix_from_network(network).iter().enumerate() {
        //     print!("{:3} ", r_index);
        //     print!("{:2}-{:2}", label[r_index].0, label[r_index].1);
        //     let mut sum: usize = 0;
        //     for &element in row {
        //         if element {
        //             print!("█");
        //             sum += 1;
        //         } else {
        //             print!("▏");
        //         }
        //     }
        //     print!(" {:2}/{:2}", sum, row.len());
        //     println!();
        // }
    }
    // }
}

#[allow(dead_code)]
pub fn log_state_matrix_wo_w2w(config: &Config, network: &Network) {
    if config.debug.log_state_matrix {
        let label = state_matrix_label_wo_w2w_from_network(network);
        for (r_index, row) in state_matrix_wo_w2w_from_network(network).iter().enumerate() {
            print!("{:3} ", r_index);
            print!("{:2}-{:2}", label[r_index].0, label[r_index].1);
            let mut sum: usize = 0;
            for &element in row {
                if element {
                    print!("█");
                    sum += 1;
                } else {
                    print!("▏");
                }
            }
            print!(" {:2}/{:2}", sum, row.len());
            println!();
        }
    }
}

/// SD の使用割合を取得する関数
pub fn get_sd_usage_ratio(network: &Network, sd: &SD, demand_list: &[Demand]) -> f64 {
    // SD が含まれているパスの数
    let mut count = 0;
    // 全体のパスの数
    let total = demand_list.len();

    // 各 Demand に対して SD が含まれているか確認
    for demand in demand_list {
        // fiber_route を取得
        let sub_routes = enumerate_subsequences(&demand.fiber_ids, MIN_BYPASS_LEN, Some(MAX_BYPASS_LEN));
        
        for sub_route in sub_routes {
            let first_fiber = network.get_fiber_by_id(sub_route.first().unwrap());
            let last_fiber  = network.get_fiber_by_id(sub_route.last().unwrap());

            let start_node = first_fiber.edge.src;
            let end_node = last_fiber.edge.dst;

            // SD の始端と終端が一致するか確認
            if sd.src == start_node && sd.dst == end_node {
                count += 1;
                break;
            }
        }
    }

    // 使用割合を計算
    if total > 0 {
        (count as f64 / total as f64) * 100.0
    } else {
        0.0
    }
}

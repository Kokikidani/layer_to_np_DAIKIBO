use rand::Rng;

use crate::{
    config::Config,
    controller::expander::{self},
    debugger, demand,
    network::{FiberID, Network, XCType},
    np_core::parameters::MAX_BYPASS_LEN,
    topology::{get_fixed_shortest_path, get_random_shortest_path, get_shortet_paths, Topology},
    Edge, SD,
};

use super::{
    ctrl_utils::{assign_all_paths, delete_all_paths},
    expander::get_min_expand_route_cand,
    output,
};
pub(super) mod iterative_designer;

pub(super) mod wxc_wbxc_designer;
pub fn main(config: &Config, initial_xc_types: &[XCType]) -> (Network, Topology, String) {
    // `initial_xc_types` を Vec<XCType> に変換して操作可能にする
    let mut xc_types = initial_xc_types.to_vec();

    // 出力ディレクトリの作成
    let output_dir: &str = &output::init_output_dir_wo_suffix(config);

    // Config情報 + コネクションの保存
    output::save_config(config, output_dir);
    output::save_connection(output_dir);

    // トポロジの取得
    let topology = Topology::new(config);

    // ネットワーク
    let mut network = Network::new(config, &topology, &xc_types);
    network.update_layer_topologies(topology.route_candidates.clone(), &[]);

    // パス需要
    let mut demand_list = demand::get_demand_list(config, &topology);

    // パス割当 (WXC-based NWの作成)
    assign_all_paths(config, &mut network, &topology, &mut demand_list);

    // 従来手法における結果を記録
    let conv_nw_w2w_fiber_count = *network
        .get_fiber_breakdown()
        .get(&[XCType::Wxc, XCType::Wxc])
        .unwrap_or(&0);
    println!("conv_nw_w2w_fiber_count:{:?}", conv_nw_w2w_fiber_count);
    if xc_types == vec![XCType::Wxc, XCType::Sxc] {
        debugger::log_net_analysis(config, &network, conv_nw_w2w_fiber_count, &demand_list);
    } else {
        debugger::log_analysis(config, &network, conv_nw_w2w_fiber_count, &demand_list);
    }

    output::save_conv_output(output_dir, &network, &demand_list);

    // // println!("aaaaaaaaaaaaaaaaaa");

    // let adjacency_file = r"C:\Users\kidanikouki\OneDrive - 国立大学法人東海国立大学機構\laboratory\layer_to_np-DAIKIBO\files\topology\jpn25.txt";

    // // Python スクリプトを呼び出す
    // // Python スクリプトの絶対パス
    // let python_script = r"C:\Users\kidanikouki\OneDrive - 国立大学法人東海国立大学機構\laboratory\layer_to_np-DAIKIBO\src\controller\generate_graph.py";

    // // Python スクリプトを呼び出す
    // let output = Command::new("py")
    //     .arg(python_script)
    //     .arg(&adjacency_file)
    //     .output()
    //     .expect("Failed to execute Python script");

    // if output.status.success() {
    //     println!("Python script executed successfully.");
    //     println!("Output: {}", String::from_utf8_lossy(&output.stdout));
    // } else {
    //     eprintln!("Python script failed.");
    //     eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
    // }

    let mut taboo_list: Vec<SD> = vec![];

    // loop {
    //     // まとめることのできるパスを探す
    //     let target_edges = {
    //         if let Some(sd) = expander::find_frequently_emerge_sub_routes_sd_with_xc_types(&network, &demand_list, &taboo_list, &xc_types) {
    //             let route_cand = if xc_types == vec![XCType::Wxc, XCType::Sxc] {
    //                 let route_cands = get_shortet_paths(&topology, &sd, None);
    //                 get_min_expand_route_cand(&network, &route_cands)
    //             } else if config.network.fiber_unification {
    //                 get_fixed_shortest_path(&topology, &sd, None)
    //             } else {
    //                 get_random_shortest_path(&topology, &sd, network.rng.gen_range(0..u64::MAX), None)
    //             };

    //             if route_cand.edge_route.len() == 1 {
    //                 taboo_list.push(sd);
    //                 continue;
    //             }
    //             Some(route_cand.edge_route)
    //         } else {
    //             None
    //         }
    //     };
    //     println!("target_edges:{:?}",target_edges);

    //     if target_edges.is_none() {
    //         debugger::log_alert("There is no candidate for bypass; because all of candidates are regarded as taboo.");
    //         debugger::log_alert("Fiber increase ratio is too high.");
    //         break;
    //     }
    //     // print!("{:?}", target_edges);

    //     // 全てのパスを削除 + バイパス元を削除 + バイパス設置 + 全てのパスを再配置
    //     delete_all_paths(&mut network, &mut demand_list);
    //     expander::remove_fibers_by_edges(config, &mut network, &target_edges.clone().unwrap());
    //     expander::expand_fibers_with_xc_types(config, &mut network, &target_edges.unwrap(), &xc_types);
    //     assign_all_paths(config, &mut network, &topology, &mut demand_list);

    //     // 空のファイバを削除
    //     if xc_types.contains(&XCType::Fxc) || xc_types.contains(&XCType::Sxc) {
    //         network.delete_empty_fibers_core(config, &mut taboo_list);
    //     } else if xc_types.contains(&XCType::Wbxc) {
    //         network.delete_empty_fibers_wb(config, &mut taboo_list);
    //     } else {
    //         unimplemented!();
    //     }

    //     if xc_types == vec![XCType::Wxc, XCType::Sxc] {
    //         debugger::log_net_analysis(config, &network, conv_nw_w2w_fiber_count, &demand_list);
    //     } else {
    //         debugger::log_analysis(config, &network, conv_nw_w2w_fiber_count, &demand_list);
    //     }

    //     network.update_layer_topologies(topology.route_candidates.clone(), &demand_list);

    //     let count_ratio = debugger::analysis::calc_fiber_count_ratio(&network, conv_nw_w2w_fiber_count);
    //     if count_ratio.is_finite() && count_ratio > 1.0 + config.network.fiber_increase_rate_limit {
    //         break;
    //     }
    // }

    for bypass_len in 2..=MAX_BYPASS_LEN {
        let mut sds = expander::find_emerge_sub_routes_sd_with_xc_types_with_len(
            &network,
            &demand_list,
            &taboo_list,
            &xc_types,
            bypass_len,
        );

        loop {
            if sds.is_empty() {
                println!("❌ sds が空になったため終了");
                break;
            }

            let mut working_network = network.clone();
            let mut working_demand_list = demand_list.clone();

            delete_all_paths(&mut working_network, &mut working_demand_list);

            for sd in &sds {
                let route_cand = if xc_types == vec![XCType::Wxc, XCType::Sxc] {
                    let route_cands = get_shortet_paths(&topology, sd, None);
                    get_min_expand_route_cand(&working_network, &route_cands)
                } else if config.network.fiber_unification {
                    get_fixed_shortest_path(&topology, sd, None)
                } else {
                    get_random_shortest_path(
                        &topology,
                        sd,
                        working_network.rng.gen_range(0..u64::MAX),
                        None,
                    )
                };

                if route_cand.edge_route.len() <= 1 {
                    continue;
                }

                let target_edges = route_cand.edge_route.clone();

                expander::remove_fibers_by_edges(config, &mut working_network, &target_edges);
                expander::expand_fibers_with_xc_types(
                    config,
                    &mut working_network,
                    &target_edges,
                    &xc_types,
                );
            }

            assign_all_paths(
                config,
                &mut working_network,
                &topology,
                &mut working_demand_list,
            );

            // 空ファイバ削除
            if xc_types.contains(&XCType::Fxc) || xc_types.contains(&XCType::Sxc) {
                working_network.delete_empty_fibers_core(config, &mut taboo_list);
            } else if xc_types.contains(&XCType::Wbxc) {
                working_network.delete_empty_fibers_wb(config, &mut taboo_list);
            } else {
                unimplemented!();
            }

            // ログ出力
            if xc_types == vec![XCType::Wxc, XCType::Sxc] {
                debugger::log_net_analysis(
                    config,
                    &working_network,
                    conv_nw_w2w_fiber_count,
                    &working_demand_list,
                );
            } else {
                debugger::log_analysis(
                    config,
                    &working_network,
                    conv_nw_w2w_fiber_count,
                    &working_demand_list,
                );
            }

            let count_ratio = debugger::analysis::calc_fiber_count_ratio(
                &working_network,
                conv_nw_w2w_fiber_count,
            );

            if count_ratio.is_finite()
                && count_ratio < 1.0 + config.network.fiber_increase_rate_limit
            {
                network = working_network;
                demand_list = working_demand_list;
                sds.clear();
                break;
            } else {
                sds.pop();
            }
        }
    }
    
    output::save_output(config, output_dir, &network, &demand_list);
    output::save_taboo_list(output_dir, &taboo_list);

    delete_all_paths(&mut network, &mut demand_list);

    (network, topology, output_dir.to_string())
}

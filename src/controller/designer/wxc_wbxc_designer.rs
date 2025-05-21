use rand::Rng;

use crate::{config::Config, controller::{ctrl_utils::{assign_all_paths, delete_all_paths}, expander, output}, debugger, demand, network::{Network, XCType}, topology::{get_random_shortest_path, Topology}, SD};

pub fn main(config: &Config) -> (Network, Topology, String) {

    // 出力先ディレクトリの作成
    let output_dir: &str = &output::init_output_dir_wo_suffix(config);

    // Configと接続情報の保存
    output::save_config(config, output_dir);
    output::save_connection(output_dir);

    // 物理トポロジの取得
    let topology = Topology::new(config);

    // XC_TYPESの宣言
    let xc_types = [XCType::Wxc, XCType::Wbxc];

    // ネットワークの取得
    let mut network = Network::new(config, &topology, &xc_types);
    
    // パス需要
    let mut demand_list = demand::get_demand_list(config, &topology);

    // 従来NW (WXC only)の作成
    assign_all_paths(config, &mut network, &topology, &mut demand_list);

    // 従来NWの情報を記録
    let conv_nw_w2w_fiber_count = *network.get_fiber_breakdown().get(&[XCType::Wxc, XCType::Wxc]).unwrap_or(&0);
    debugger::log_analysis(config, &network, conv_nw_w2w_fiber_count, &demand_list);
    output::save_conv_output(output_dir, &network, &demand_list);

    // 設立後埋まらなかった区間をタブーに追加
    let mut taboo_list: Vec<SD> = vec![];

    // // レイヤ化NWの設計
    loop {
        // まとめられるパスを探す
        let target_edge_route = {
            let sd = expander::find_frequently_emerge_sub_routes_sd_with_xc_types(&network, &demand_list, &taboo_list, &xc_types).unwrap();
            let route_candidate = get_random_shortest_path(&topology, &sd, network.rng.gen_range(0..u64::MAX), None);
            if route_candidate.edge_route.len() == 1 {
                taboo_list.push(sd);
                debugger::log_taboo_list_addition(config, &sd);
                continue;
            }
            route_candidate.edge_route
        };

        // 全てのパスを削除 + バイパスファイバ配置 + 全てのパスを再配置
        delete_all_paths(&mut network, &mut demand_list);
        expander::expand_fibers_with_xc_types(config, &mut network, &target_edge_route, &[XCType::Wxc, XCType::Wbxc]);
        assign_all_paths(config, &mut network, &topology, &mut demand_list);

        // 使用していないファイバを削除
        network.delete_empty_fibers_wb(config, &mut taboo_list);
        network.delete_empty_fibers(config, &mut taboo_list);

        debugger::log_analysis(config, &network, conv_nw_w2w_fiber_count, &demand_list);

        // 終了判定
        let count_ratio = debugger::analysis::calc_fiber_count_ratio(&network, conv_nw_w2w_fiber_count);
        if count_ratio.is_finite() && count_ratio > 1.0 + config.network.fiber_increase_rate_limit {
            break;
        }
    }
    // loop {
    //     // まとめられるパスを探す
    //     let target_edge_routes = {
    //         // すべての SD を取得
    //         let sds = expander::find_frequently_emerge_sub_routes_sd_with_xc_types(&network, &demand_list, &taboo_list, &xc_types);

    //         // SD が見つからない場合は None を返してループを終了
    //         if sds.is_empty() {
    //             None
    //         } else {
    //             let mut all_edge_routes = Vec::new();

    //             // 各 SD に対してバイパスを設立
    //             for sd in sds {
    //                 // ランダムな最短経路を取得
    //                 let route_candidate = get_random_shortest_path(&topology, &sd, network.rng.gen_range(0..u64::MAX), None);

    //                 // バイパスが 1 つのエッジしか含まれていない場合はタブーリストに追加
    //                 if route_candidate.edge_route.len() == 1 {
    //                     taboo_list.push(sd);
    //                     debugger::log_taboo_list_addition(config, &sd);
    //                     continue;
    //                 }

    //                 // 複数のエッジを含む場合は、エッジルートを収集
    //                 all_edge_routes.push(route_candidate.edge_route);
    //             }

    //             // 収集したエッジルートを結合して返す
    //             if all_edge_routes.is_empty() {
    //                 None
    //             } else {
    //                 Some(all_edge_routes.concat())
    //             }
    //         }
    //     };

    //     // バイパス候補がない場合はループを終了
    //     if target_edge_routes.is_none() {
    //         debugger::log_alert("There is no candidate for bypass; because all of candidates are regarded as taboo.");
    //         debugger::log_alert("Fiber increase ratio is too high.");
    //         break;
    //     }

    //     // 全てのパスを削除 + バイパスファイバ配置 + 全てのパスを再配置
    //     delete_all_paths(&mut network, &mut demand_list);
    //     expander::expand_fibers_with_xc_types(config, &mut network, &target_edge_routes.clone().unwrap(), &[XCType::Wxc, XCType::Wbxc]);
    //     assign_all_paths(config, &mut network, &topology, &mut demand_list);

    //     // 使用していないファイバを削除
    //     network.delete_empty_fibers_wb(config, &mut taboo_list);
    //     network.delete_empty_fibers(config, &mut taboo_list);

    //     // 分析結果をログ出力
    //     debugger::log_analysis(config, &network, conv_nw_w2w_fiber_count, &demand_list);

    //     // 終了判定
    //     let count_ratio = debugger::analysis::calc_fiber_count_ratio(&network, conv_nw_w2w_fiber_count);
    //     if count_ratio.is_finite() && count_ratio > 1.0 + config.network.fiber_increase_rate_limit {
    //         break;
    //     }
    // }


    // 最終結果出力
    output::save_output(config, output_dir, &network, &demand_list);
    output::save_taboo_list(output_dir, &taboo_list);

    // 全てのパスを削除
    delete_all_paths(&mut network, &mut demand_list);

    (network, topology, output_dir.to_string())

}
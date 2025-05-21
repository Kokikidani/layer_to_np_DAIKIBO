use fxhash::FxHashMap;
use strum::IntoEnumIterator;
use std::collections::HashMap;
use std::{ env, fs };
 
use std::fs::{create_dir_all, File};
use std::io::Write;

use crate::network::{EdgesType,FiberType, PortID};
use crate::np_core::parameters::{CURVE_GRAPH_SCRIPT, TRAVERSE_GRAPH_SCRIPT};
use crate::utils::find_x_for_y;
use crate::{Edge, Node};
use crate::{
    config::Config,
    demand::Demand,
    network::{ Network, XCType },
    utils::{
        generate_id, get_file, output_file_from_2dvec
    }, SD,
};
 
pub fn init_master_dir(config: &mut Config) {
    // ディレクトリ名の決定
    let id: String = generate_id();
    let master_dir: String = format!(
        "./{}/{}/{}/{:.0}/{}",
        config.simulation.outdir,
        config.network.topology,
        config.network.node_configuration,
        config.simulation.traffic_intensity * 10.0,
    id);
 
    // ディレクトリの作成
    match fs::create_dir_all(&master_dir) {
        Ok(_) => (),
        Err(_) => panic!("ディレクトリの作成に失敗しました (権限?)"),
    }
 
    config.simulation.outdir = master_dir;
}
 
/// todo: change output to `std::path::Path`
pub fn init_output_dir_wo_suffix(config: &Config) -> String {
    // ディレクトリ名の決定
    let output_dir: String = format!("./{}", config.simulation.outdir);
 
    // ディレクトリの作成
    match fs::create_dir_all(&output_dir) {
        Ok(_) => (),
        Err(_) => panic!("ディレクトリの作成に失敗しました (権限?)"),
    }
 
    // ディレクトリ名を返却
    output_dir
}
 
pub fn save_connection(output_dir: &str) {
    let datas = match env::var("SSH_CONNECTION") {
        Ok(data) => {
            let tmp_datas: Vec<&str> = data.split_whitespace().collect();
 
            let client_ip = tmp_datas[0].to_string();
            let client_port = tmp_datas[1].to_string();
            let server_ip = tmp_datas[2].to_string();
            let server_port = tmp_datas[3].to_string();
 
            vec![client_ip, client_port, server_ip, server_port]
        }
        Err(_err) => { vec!["LOCAL".to_string()] }
    };
 
    let contents = format!("{:?}", datas);
 
    match fs::write(format!("{}/ssh_connection.txt", output_dir), contents) {
        Ok(_) => (),
        Err(_) => panic!("ファイル書き込みに失敗しました"),
    }
}
 
/// config構造体をファイルとして出力する
/// config構造体->TOMLデータ->文字列->ファイル
pub fn save_config(config: &Config, output_dir: &str) {
    match toml::Value::try_from(config) {
        Ok(toml_data) =>
            match toml::to_string_pretty(&toml_data) {
                Ok(toml_string) => {
                    match fs::write(format!("{}/config.toml", output_dir), toml_string) {
                        Ok(_) => (),
                        Err(_) => panic!("TOML文字列をファイルに書き込めませんでした"),
                    }
                }
                Err(_) => panic!("TOMLデータを文字列に変換できませんでした"),
            }
        Err(err) => panic!("構造体をTOMLデータに変換できませんでした: {}\n{:?}", err, config),
    }
}
 
pub fn save_best(output_dir: &str, best_str: &str) {
    let _f: File = get_file(&format!("{}/0_best_is_{}.txt", output_dir, best_str));
}
 
pub fn save_taboo_list(output_dir: &str, taboo_list: &[SD]) {
    let mut file: File = get_file(&format!("{}/taboo_list.txt", output_dir));
    for t in taboo_list {
        writeln!(file, "{t}").unwrap();
    }
}
 
pub fn save_conv_output(
    output_dir: &str,
    network: &Network,
    demand_list: &[Demand]
) {
    create_dir_all(format!("{output_dir}/conv/")).unwrap();
    save_analytics(&format!("{output_dir}/conv/"), network, demand_list);
 
    // let filename_prefix = format!("{}/conv", format!("{output_dir}"));
    // let fiber_label = network.get_fiber_output();
    // let _ = output_file_from_vec(&format!("{}_fiber_label.csv", filename_prefix), &fiber_label);
}
 
/// traffic_intensityの整数値ごとに出力すること．
/// demand_listはスライスして載せましょう
/// ファイル出力
/// - 各種ファイバの本数
/// - 横軸WXCポート通過回数，縦軸demandパス比率となる配列
/// - 横軸FXCポート通過回数，縦軸demandパス比率となる配列
/// - パス収容率
/// - 各パスのWXC/FXCポート通過回数
pub fn save_output(
    config: &Config,
    output_dir: &str,
    network: &Network,
    demand_list: &[Demand]
) {
    create_dir_all(format!("{output_dir}/prop/")).unwrap();
    save_analytics(&format!("{output_dir}/prop/"), network, demand_list);
    save_wxc_port_pass_count_img(config);
    save_add_drop_count(output_dir, network,demand_list);
    save_specific_fiber_info(output_dir, network, demand_list);
    save_transition_counts_around_node(output_dir, network, demand_list);
    save_transition_counts_with_device_info(output_dir, network, demand_list);
    save_specific_fiber_info_with_ids(output_dir, network, demand_list,);
    save_transition_counts_with_slots(output_dir, network, demand_list);
    
    // ファイバ配置情報
    // let fiber_label = network.get_fiber_output();
    // let _ = output_file_from_vec(&format!("{}_fiber_label.csv", filename_prefix), &fiber_label);
 
    // 各Edgeの出力
    let edges = network.get_edges_advanced_double();
 
    let filename_prefix = format!("{}/", output_dir);
 
    let _ = output_file_from_2dvec(&format!("{}_edges_advanced.txt", filename_prefix), &edges);
}

pub fn save_grooming_output(
    config: &Config,
    output_dir: &str,
    network: &Network,
    demand_list: &[Demand]
) {
    create_dir_all(format!("{output_dir}/grooming/")).unwrap();
    save_analytics(&format!("{output_dir}/grooming/"), network, demand_list);
}

pub fn save_specific_fiber_info(output_dir: &str, network: &Network, demand_list: &[Demand]) {
    let target_node_id = Node::new(18);  // ノードID 18をターゲットとする
    let file_path = format!("{}/node_{}_wxc_paths.txt", output_dir, target_node_id);
    let mut file = File::create(&file_path).expect("Failed to create file");

    writeln!(file, "Paths with Add/Drop transitions for node {}:", target_node_id).unwrap();

    for demand in demand_list {
        let mut path_info = Vec::new();
        let mut includes_target_node = false;

        // 各ファイバの src, dst ノードと XCType を取得
        for (index, fiber_id) in demand.fiber_ids.iter().enumerate() {
            let fiber = network.get_fiber_by_id(fiber_id);
            let [src_type, dst_type] = network.get_fiber_sd_xc_type(fiber);

            // 最初のノードは Add、最後のノードは Drop、その他は in/out として追加
            if index == 0 {
                path_info.push((fiber.edge.src, "Add", src_type));
            } else {
                path_info.push((fiber.edge.src, "out", src_type));
            }

            if index == demand.fiber_ids.len() - 1 {
                path_info.push((fiber.edge.dst, "Drop", dst_type));
            } else {
                path_info.push((fiber.edge.dst, "in", dst_type));
            }

            // ターゲットノードが含まれているかをチェック
            if fiber.edge.src == target_node_id || fiber.edge.dst == target_node_id {
                includes_target_node = true;
            }
        }

        // 指定ノード ID を含むパスのみを出力
        if includes_target_node {
            for (idx, (node, direction, xc_type)) in path_info.iter().enumerate() {
                if idx > 0 {
                    write!(file, " > ").unwrap();
                }
                write!(file, "{}{}({:?})", node, direction, xc_type).unwrap();
            }
            writeln!(file).unwrap();
        }
    }
}

pub fn save_specific_fiber_info_with_ids(output_dir: &str, network: &Network, demand_list: &[Demand]) {
    let target_node_id = Node::new(18); // ノードID 18をターゲットとする
    let file_path = format!("{}/node_{}_wxc_demand_ids.txt", output_dir, target_node_id);
    let mut file = File::create(&file_path).expect("Failed to create file");

    writeln!(file, "Demands passing through node {} WXC:", target_node_id).unwrap();

    for demand in demand_list {
        let mut path_info = Vec::new();
        let mut includes_target_node = false;

        // 各ファイバの src, dst ノードと XCType を取得
        for (index, fiber_id) in demand.fiber_ids.iter().enumerate() {
            let fiber = network.get_fiber_by_id(fiber_id);
            let [src_type, dst_type] = network.get_fiber_sd_xc_type(fiber);

            // 最初のノードは Add、最後のノードは Drop、その他は in/out として追加
            if index == 0 {
                path_info.push((fiber.edge.src, "Add", src_type));
            } else {
                path_info.push((fiber.edge.src, "out", src_type));
            }

            if index == demand.fiber_ids.len() - 1 {
                path_info.push((fiber.edge.dst, "Drop", dst_type));
            } else {
                path_info.push((fiber.edge.dst, "in", dst_type));
            }

            // ターゲットノードが含まれているかをチェック
            if fiber.edge.src == target_node_id || fiber.edge.dst == target_node_id {
                includes_target_node = true;
            }
        }

        // 指定ノード ID を含むパスのみを出力
        if includes_target_node {
            writeln!(file, "Demand ID: {}", demand.index).unwrap();
            for (idx, (node, direction, xc_type)) in path_info.iter().enumerate() {
                if idx > 0 {
                    write!(file, " > ").unwrap();
                }
                write!(file, "{}{}({:?})", node, direction, xc_type).unwrap();
            }
            writeln!(file).unwrap();
        }
    }
}


pub fn save_transition_counts_around_node(output_dir: &str, network: &Network, demand_list: &[Demand]) {
    let target_node_id = Node::new(16);  // ノードID 14をターゲットとする
    let file_path = format!("{}/transitions_around_node_{}_counts.txt", output_dir, target_node_id);
    let mut file = File::create(&file_path).expect("Failed to create file");

    writeln!(file, "Transition counts around node {}:", target_node_id).unwrap();

    // 遷移ごとのカウントを保存するハッシュマップ
    let mut transition_counts: HashMap<String, usize> = HashMap::new();

    for demand in demand_list {
        let mut path_info = Vec::new();

        // 各ファイバの src, dst ノードと XCType を取得し、最初と最後をAdd/Drop、それ以外はWxcで追加
        for (index, fiber_id) in demand.fiber_ids.iter().enumerate() {
            let fiber = network.get_fiber_by_id(fiber_id);
            let [src_type, dst_type] = network.get_fiber_sd_xc_type(fiber);

            if index == 0 {
                path_info.push((fiber.edge.src, "Add", src_type));
            } else {
                path_info.push((fiber.edge.src, "Wxc", src_type));
            }

            if index == demand.fiber_ids.len() - 1 {
                path_info.push((fiber.edge.dst, "Drop", dst_type));
            } else {
                path_info.push((fiber.edge.dst, "Wxc", dst_type));
            }
        }

        // ノード14が含まれる位置を探し、その前後の遷移をカウント
        let mut i = 0;
        while i < path_info.len() {
            if path_info[i].0 == target_node_id {
                let transition;

                // 連続する14を無視し、次の異なるノードを取得
                let mut j = i;
                while j + 1 < path_info.len() && path_info[j + 1].0 == target_node_id {
                    j += 1;
                }

                // ノード14でAddの場合は次のノードも含める
                if path_info[i].1 == "Add" {
                    if j + 1 < path_info.len() {
                        let (next_node, _, next_xc_type) = path_info[j + 1];
                        transition = format!(
                            "{}(Add) > {}({:?}) > {}({:?})",
                            target_node_id, target_node_id, path_info[i].2,
                            next_node, next_xc_type
                        );
                    } else {
                        i = j + 1;
                        continue;
                    }
                }
                // ノード14でDropの場合は前のノードも含める
                else if path_info[i].1 == "Drop" {
                    if i > 0 {
                        let (prev_node, _, prev_xc_type) = path_info[i - 1];
                        transition = format!(
                            "{}({:?}) > {}({:?}) > {}(Drop)",
                            prev_node, prev_xc_type,
                            target_node_id, path_info[i].2,
                            target_node_id
                        );
                    } else {
                        i = j + 1;
                        continue;
                    }
                }
                // ノード14でAdd/Dropがない場合は前後のノードを含む
                else if i > 0 && j + 1 < path_info.len() {
                    let (prev_node, _, prev_xc_type) = path_info[i - 1];
                    let (next_node, _, next_xc_type) = path_info[j + 1];
                    transition = format!(
                        "{}({:?}) > {}({:?}) > {}({:?})",
                        prev_node, prev_xc_type,
                        path_info[i].0, path_info[i].2,
                        next_node, next_xc_type
                    );
                } else {
                    i = j + 1;
                    continue;
                }

                // カウントを更新
                *transition_counts.entry(transition).or_insert(0) += 1;

                // 連続する `14` をスキップ
                i = j;
            }
            i += 1;
        }
    }

    // 結果を整列
    // 結果を整列
    let mut sorted_transitions: Vec<_> = transition_counts.iter().collect();

    sorted_transitions.sort_by(|a, b| {
        let parse_transition = |trans: &str| {
            let parts: Vec<_> = trans.split_whitespace().collect();
            let first = parts[0].split('(').next().unwrap().parse::<usize>().unwrap_or(0);
            let second_type = parts[1].split('(').next().unwrap().to_string(); // Stringに変換
            let third = parts[2].split('(').next().unwrap().parse::<usize>().unwrap_or(0);
            (first, second_type, third)
        };
    
        let (a_first, a_second_type, a_third) = parse_transition(a.0);
        let (b_first, b_second_type, b_third) = parse_transition(b.0);
    
        a_first.cmp(&b_first)
            .then_with(|| {
                // "Wxc"が"Fxc"よりも先に来るようにカスタム比較
                match (a_second_type.as_str(), b_second_type.as_str()) {
                    ("Wxc", "Fxc") => std::cmp::Ordering::Less,
                    ("Fxc", "Wxc") => std::cmp::Ordering::Greater,
                    _ => a_second_type.cmp(&b_second_type),
                }
            })
            .then_with(|| a_third.cmp(&b_third))
    });
    

    // ファイルに出力
    for (transition, count) in sorted_transitions {
        writeln!(file, "{} : {}", transition, count).unwrap();
    }
}

pub fn save_transition_counts_with_device_info(
    output_dir: &str,
    network: &Network,
    demand_list: &[Demand],
) {
    let target_node_id = Node::new(13);
    let file_path = format!("{}/transitions_with_device_info_{}_counts.txt", output_dir, target_node_id);
    let mut file = File::create(&file_path).expect("Failed to create file");

    writeln!(file, "Transition counts with device info around node {}:", target_node_id).unwrap();

    let mut transition_counts: HashMap<String, usize> = HashMap::new();

    for demand in demand_list {
        let mut path_info = Vec::new();

        for (index, fiber_id) in demand.fiber_ids.iter().enumerate() {
            let fiber = network.get_fiber_by_id(fiber_id);
            let [src_type, dst_type] = network.get_fiber_sd_xc_type(fiber);

            // fiber_idからsrc_port_idとdst_port_idを取得
            let src_port_id = &fiber.src_port_ids;
            let dst_port_id = &fiber.dst_port_ids;

            // path_infoに追加（Add、Wxc、Dropの情報をsrc_port_id、dst_port_idと共に保存）
            if index == 0 {
                path_info.push((fiber.edge.src, "Add", src_type, src_port_id));
            } else {
                path_info.push((fiber.edge.src, "Wxc", src_type, src_port_id));
            }

            if index == demand.fiber_ids.len() - 1 {
                path_info.push((fiber.edge.dst, "Drop", dst_type, dst_port_id));
            } else {
                path_info.push((fiber.edge.dst, "Wxc", dst_type, dst_port_id));
            }
        }

        let mut i = 0;
        while i < path_info.len() {
            if path_info[i].0 == target_node_id {
                let transition;

                let mut j = i;
                while j + 1 < path_info.len() && path_info[j + 1].0 == target_node_id {
                    j += 1;
                }

                if path_info[i].1 == "Add" {
                    if j + 1 < path_info.len() {
                        let (next_node, _, _next_xc_type, next_port_id) = path_info[j + 1];
                        transition = format!(
                            "{}(Add) > {}(Wxc_in:{:?}_out:{:?}) > {}(Wxc_in:{:?})",
                            target_node_id,
                            target_node_id, path_info[i].3, next_port_id, // ノード14のinポートとoutポート
                            next_node, next_port_id
                        );
                    } else {
                        i = j + 1;
                        continue;
                    }
                } else if path_info[i].1 == "Drop" {
                    if i > 0 {
                        let (prev_node, _, prev_xc_type, prev_port_id) = path_info[i - 1];
                        transition = format!(
                            "{}({}_{:?}) > {}(Wxc_in:{:?}) > {}(Drop)",
                            prev_node, prev_xc_type, prev_port_id, // 前のノードのポート
                            target_node_id, path_info[i].3, // ノード14のinポート
                            target_node_id
                        );
                    } else {
                        i = j + 1;
                        continue;
                    }
                } else if i > 0 && j + 1 < path_info.len() {
                    let (prev_node, _, prev_xc_type, prev_port_id) = path_info[i - 1];
                    let (next_node, _, next_xc_type, next_port_id) = path_info[j + 1];
                    transition = format!(
                        "{}({}_{:?}) > {}(Wxc_in:{:?}_out:{:?}) > {}({}_{:?})",
                        prev_node, prev_xc_type, prev_port_id, // 前のノードのポート
                        path_info[i].0, path_info[i].3, next_port_id, // ノード14のinポートとoutポート
                        next_node, next_xc_type, next_port_id // 次のノードのポート
                    );
                } else {
                    i = j + 1;
                    continue;
                }

                *transition_counts.entry(transition).or_insert(0) += 1;
                i = j;
            }
            i += 1;
        }
    }

    let mut sorted_transitions: Vec<_> = transition_counts.iter().collect();
    sorted_transitions.sort_by(|a, b| a.0.cmp(b.0));

    for (transition, count) in sorted_transitions {
        writeln!(file, "{} : {}", transition, count).unwrap();
    }
}

pub fn save_transition_counts_with_slots(
    output_dir: &str,
    network: &Network,
    demand_list: &[Demand],
) {
    let target_node_id = Node::new(13);
    let file_path = format!("{}/transitions_with_slots_{}_counts.txt", output_dir, target_node_id);
    let mut file = File::create(&file_path).expect("Failed to create file");

    writeln!(file, "Transition counts with slots info around node {}:", target_node_id).unwrap();

    let mut transition_counts: HashMap<String, usize> = HashMap::new();

    for demand in demand_list {
        let mut path_info = Vec::new();

        // スロット情報の取得
        let mut slots_iter = demand.slot_heads.iter();

        for (index, fiber_id) in demand.fiber_ids.iter().enumerate() {
            let fiber = network.get_fiber_by_id(fiber_id);
            let [src_type, dst_type] = network.get_fiber_sd_xc_type(fiber);

            // fiber_id から src_port_id と dst_port_id を取得
            let src_port_id = &fiber.src_port_ids;
            let dst_port_id = &fiber.dst_port_ids;

            // 対応するスロット番号を取得
            let slot = slots_iter.next().unwrap_or(&0);

            // path_info に追加（Add、Wxc、Dropの情報をスロット番号と共に保存）
            if index == 0 {
                path_info.push((fiber.edge.src, "Add", src_type, src_port_id, *slot));
            } else {
                path_info.push((fiber.edge.src, "Wxc", src_type, src_port_id, *slot));
            }

            if index == demand.fiber_ids.len() - 1 {
                path_info.push((fiber.edge.dst, "Drop", dst_type, dst_port_id, *slot));
            } else {
                path_info.push((fiber.edge.dst, "Wxc", dst_type, dst_port_id, *slot));
            }
        }

        let mut i = 0;
        while i < path_info.len() {
            if path_info[i].0 == target_node_id {
                let transition;

                let mut j = i;
                while j + 1 < path_info.len() && path_info[j + 1].0 == target_node_id {
                    j += 1;
                }

                if path_info[i].1 == "Add" {
                    if j + 1 < path_info.len() {
                        let (next_node, _, _next_xc_type, next_port_id, next_slot) = path_info[j + 1];
                        transition = format!(
                            "{}(Add_slot:{}) > {}(Wxc_slot:{}_in:{:?}_out:{:?}) > {}(Wxc_slot:{})",
                            target_node_id,
                            path_info[i].4,
                            target_node_id,
                            path_info[i].4, path_info[i].3, next_port_id, // ノード13のinポートとoutポート
                            next_node, next_slot
                        );
                    } else {
                        i = j + 1;
                        continue;
                    }
                } else if path_info[i].1 == "Drop" {
                    if i > 0 {
                        let (prev_node, _, prev_xc_type, prev_port_id, prev_slot) = path_info[i - 1];
                        transition = format!(
                            "{}({}_{:?}_slot:{}) > {}(Wxc_slot:{}) > {}(Drop_slot:{})",
                            prev_node, prev_xc_type, prev_port_id, prev_slot, // 前のノードのポート
                            target_node_id, path_info[i].4, // ノード13のスロット
                            target_node_id, path_info[i].4
                        );
                    } else {
                        i = j + 1;
                        continue;
                    }
                } else if i > 0 && j + 1 < path_info.len() {
                    let (prev_node, _, prev_xc_type, prev_port_id, prev_slot) = path_info[i - 1];
                    let (next_node, _, next_xc_type, next_port_id, next_slot) = path_info[j + 1];
                    transition = format!(
                        "{}({}_{:?}_slot:{}) > {}(Wxc_slot:{}_in:{:?}_out:{:?}) > {}({}_{:?}_slot:{})",
                        prev_node, prev_xc_type, prev_port_id, prev_slot, // 前のノードのポートとスロット
                        path_info[i].0, path_info[i].4, path_info[i].3, next_port_id, // ノード13のinポートとoutポート
                        next_node, next_xc_type, next_port_id, next_slot // 次のノードのポートとスロット
                    );
                } else {
                    i = j + 1;
                    continue;
                }

                *transition_counts.entry(transition).or_insert(0) += 1;
                i = j;
            }
            i += 1;
        }
    }

    let mut sorted_transitions: Vec<_> = transition_counts.iter().collect();
    sorted_transitions.sort_by(|a, b| a.0.cmp(b.0));

    for (transition, count) in sorted_transitions {
        writeln!(file, "{} : {}", transition, count).unwrap();
    }
}



pub fn save_add_drop_count(output_dir: &str, network: &Network, demand_list: &[Demand]) {
    let mut add_count = FxHashMap::default();
    let mut drop_count = FxHashMap::default();
    let mut wxc_to_wxc_count = 0;

    // 各パスの add/drop と Wxc->Wxc の数をカウント
    for demand in demand_list {
        // パスの始点と終点のみを使用
        if let Some(first_fiber_id) = demand.fiber_ids.first() {
            let first_fiber = network.get_fiber_by_id(first_fiber_id);
            *add_count.entry(first_fiber.edge.src).or_insert(0) += 1;
        }
        if let Some(last_fiber_id) = demand.fiber_ids.last() {
            let last_fiber = network.get_fiber_by_id(last_fiber_id);
            *drop_count.entry(last_fiber.edge.dst).or_insert(0) += 1;
        }

        // 各ファイバの Wxc -> Wxc 移動をカウント
        for fiber_id in &demand.fiber_ids {
            let fiber = network.get_fiber_by_id(fiber_id);
            let [src_type, dst_type] = network.get_fiber_sd_xc_type(fiber);

            if src_type == XCType::Wxc && dst_type == XCType::Wxc {
                wxc_to_wxc_count += 1;
            }
        }
    }

    // add/drop 数の合計を計算
    let total_add_drop: usize = add_count.values().sum::<usize>() + drop_count.values().sum::<usize>();

    // 結果を count.txt に出力
    let mut f = File::create(format!("{}/count.txt", output_dir)).expect("Failed to create count.txt");
    writeln!(f, "WxcからWxcの移動数: {}", wxc_to_wxc_count).unwrap();
    writeln!(f, "Add/drop数: {}", total_add_drop).unwrap();
}
 
fn calc_wxc_pass_count(network: &Network, demand_list: &[Demand]) -> Vec<usize> {
    let mut wxc_pass_count = vec![];
 
    for demand in demand_list {
        let mut count = 0;
        for fiber_id in &demand.fiber_ids {
            let [src_type, dst_type] = network.get_fiber_sd_xc_type_by_id(fiber_id);
            
            match src_type{
                XCType::Wxc  => count += 1,
                XCType::Wbxc | XCType::Fxc | XCType::Sxc => (),
                XCType::Added_Wxc => todo!(),
            }
            match dst_type{
                XCType::Wxc  => count += 1,
                XCType::Wbxc | XCType::Fxc | XCType::Sxc => (),
                XCType::Added_Wxc => todo!(),
            }
        }
 
        while wxc_pass_count.len() < count + 1 {
            wxc_pass_count.push(0);
        }
 
        if count != 0 {
            wxc_pass_count[count] += 1;
        }
    }
 
    wxc_pass_count
}

pub fn calc_wxc_pass_count_average(network: &Network, demand_list: &[Demand]) -> f64 {
    let wxc_pass_count = calc_wxc_pass_count(network, demand_list);
 
    let mut sum_product = 0.0;
    for (count, pass_count) in wxc_pass_count.iter().enumerate() {
        sum_product += (count * *pass_count) as f64;
    }
 
    sum_product / demand_list.len() as f64
 
}
 
fn save_analytics(output_dir: &str, network: &Network, demand_list: &[Demand]) {
    save_fiber_breakdown(output_dir, network);
    save_fiber_breakdown_on_each_link(output_dir, network);
    save_wxc_port_pass_count(output_dir, network, demand_list);
    save_network_info(output_dir, network);
    save_mcf_stats(output_dir, network);
    // save_network_capacity(output_dir, x, y1, y2);
    save_path_info(output_dir, network, demand_list);
    save_xc_scale(output_dir, network);
}
 
fn get_mut_file(filepath: &str) -> File {
    File::create(filepath).unwrap()
}
 
fn save_fiber_breakdown(output_dir: &str, network: &Network) {
    let fiber_breakdown = network.get_fiber_breakdown();
    
    let mut f = get_mut_file(&format!("{output_dir}/fiber_breakdown.txt"));
    for src_type in XCType::iter() {
        for dst_type in XCType::iter() {
            let fiber_count = fiber_breakdown.get(&[src_type, dst_type]).unwrap_or(&0);
            if *fiber_count != 0 {
                writeln!(f, "{src_type}={dst_type}: {fiber_count}").unwrap();
            }
        }
    }
}
 
fn save_mcf_stats(output_dir: &str, network: &Network) {
    let mut f = get_mut_file(&format!("{output_dir}/mcf_stats.txt"));
    for edge in &network.edges {
        let fiber_ids_on_edge = network.get_fiber_id_on_edge(edge);
        for fiber_id in fiber_ids_on_edge {
            let fiber = network.get_fiber_by_id(&fiber_id);
            if fiber.fiber_type == FiberType::Mcf {
                let unused_core_num = network.get_unused_core(fiber).len();
                let core_num = fiber.get_core_num();
                let [src_xc_type, dst_xc_type] = network.get_fiber_sd_xc_type(fiber);
                writeln!(f, "{} {}={} {}/{}", edge, src_xc_type, dst_xc_type, core_num-unused_core_num, core_num).unwrap();
            }
        }
    }
}
 
fn save_fiber_breakdown_on_each_link(output_dir: &str, network: &Network) {
    let mut f = get_mut_file(&format!("{output_dir}/fiber_breakdown_on_each_link.txt"));
    for edge in &network.edges {
        let fiber_ids_on_edge = network.get_fiber_id_on_edge(edge);
        let mut counter = FxHashMap::default();
        for fiber_id in fiber_ids_on_edge {
            let fiber = network.get_fiber_by_id(&fiber_id);
            let entry = counter.entry(fiber.sd_xc_type).or_insert(0);
            *entry += 1;
        }
        write!(f, "{} ", edge).unwrap();
        for src_type in XCType::iter() {
            for dst_type in XCType::iter() {
                write!(f, "{}-{}: {:3}\t", src_type, dst_type, counter.get(&[src_type, dst_type]).unwrap_or(&0)).unwrap();
            }
        }
        writeln!(f).unwrap();
    }
}
 
pub fn save_network_info(output_dir: &str, network: &Network) {
    let export = network.export();
 
    let mut export_vec: Vec<(EdgesType, Vec<Edge>, usize)> = export.iter().map(|((edges_type, edge_route), count)| (*edges_type, edge_route.to_vec(), *count)).collect();
    export_vec.sort_by_key(|(_et, er, _c)| er.to_vec());
    export_vec.sort_by_key(|(et, _er, _c)| *et as usize);
 
    let mut f = get_mut_file(&format!("{output_dir}/network_info.txt"));
    for (et, er, c) in export_vec {
        // debug_println!(et, &er, c);
        write!(f, "{:?} | {:2} |", et, c).unwrap();
        
        write!(f, "{}", er.first().unwrap()).unwrap();
        for edge in er.iter().skip(1) {
            write!(f, " => {}", edge).unwrap();
        }
        writeln!(f).unwrap();        
    }
}
 
fn save_wxc_port_pass_count(output_dir: &str, network: &Network, demand_list: &[Demand]) {
    let mut wxc_pass_count_dist = vec![0];
    let mut wxc_pass_count_sum = 0;
 
    for demand in demand_list {
        let mut count = 0;
        for fiber_id in &demand.fiber_ids {
            let [src_type, dst_type] = network.get_fiber_sd_xc_type_by_id(fiber_id);
            
            match src_type{
                XCType::Wxc  => count += 1,
                XCType::Wbxc | XCType::Fxc | XCType::Sxc => (),
                XCType::Added_Wxc => todo!(),
            }
            match dst_type{
                XCType::Wxc  => count += 1,
                XCType::Wbxc | XCType::Fxc | XCType::Sxc => (),
                XCType::Added_Wxc => todo!(),
            }
        }
 
        while wxc_pass_count_dist.len() < count + 1 {
            wxc_pass_count_dist.push(0);
        }
 
        wxc_pass_count_dist[count] += 1;
        wxc_pass_count_sum += count;
    }
 
    let mut f = get_mut_file(&format!("{output_dir}/wxc_port_pass_count.txt"));
    for (wxc_pass_count, count) in wxc_pass_count_dist.into_iter().enumerate() {
        writeln!(f, "{wxc_pass_count:2}: {count}").unwrap();
    }
 
    let wxc_pass_count_ave = wxc_pass_count_sum as f64 / demand_list.len() as f64;
    writeln!(f, "Ave: {wxc_pass_count_ave:.5}").unwrap();
}
 
fn save_path_info(output_dir: &str, network: &Network, demand_list: &[Demand]) {
 
    // 0. 各パスの長さ (ホップ数)
    // 1. 各パスのWXC/WBXC/FXCポート通過回数
    // 2. 各パスのバイパス通過回数
    // 3. 各パスのバイパス区間割合
 
    let mut f = get_mut_file(&format!("{output_dir}/path_info.txt"));
    writeln!(f, "PATH_LEN(HOP) WXC_PORT_TRAVERSAL_COUNT WBXC_PORT_TRAVERSAL_COUNT FXC_PORT_TRAVERSAL_COUNT BYPASS_COUNT BYPASS_PROP").unwrap();
 
    for demand in demand_list {
        let path_len = demand.fiber_ids.len();
        let mut path_traversal_count = FxHashMap::default();
       
        let mut path_bypass_count = 0;
        for fiber_id in &demand.fiber_ids {
            let [src_xc_type, dst_xc_type] = network.get_fiber_sd_xc_type_by_id(fiber_id);
 
            let src_entry = path_traversal_count.entry(src_xc_type).or_insert(0);
            *src_entry += 1;
 
            let dst_entry = path_traversal_count.entry(dst_xc_type).or_insert(0);
            *dst_entry += 1;
 
            if src_xc_type == XCType::Wxc && dst_xc_type != XCType::Wxc {
                path_bypass_count += 1;
            }
        }
 
        let path_bypass_prop = 1.0 - *path_traversal_count.get(&XCType::Wxc).unwrap_or(&0) as f64 / ((path_len + 1) * 2) as f64;
 
        writeln!(f, "{} {} {} {} {} {:.5}",
            path_len,
            path_traversal_count.get(&XCType::Wxc).unwrap_or(&0),
            path_traversal_count.get(&XCType::Wbxc).unwrap_or(&0),
            path_traversal_count.get(&XCType::Fxc).unwrap_or(&0),
            path_bypass_count,
            path_bypass_prop
        ).unwrap();
    }
}
 
pub fn save_xc_scale(output_dir: &str, network: &Network) {
 
    let mut xc_scales = FxHashMap::default();
 
    for node in network.get_nodes() {
        let entry = xc_scales.entry(node).or_insert(FxHashMap::default());
        for xc_type in XCType::iter() {
            if let Some(xc_on_node) = network.get_xc_on_node(node.into(), &xc_type) {
                entry.insert(xc_type, xc_on_node.get_size());
            } else{
                entry.insert(xc_type, 0);
            }
        }
    }
 
    let mut f = get_mut_file(&format!("{output_dir}/node_scale.txt"));
    for (node, xc_scale_on_node) in xc_scales.iter() {
        write!(f, "{} ", node).unwrap();
        for (xc_type, scale) in xc_scale_on_node.iter() {
            write!(f, "{}: {:3}, ", xc_type, scale).unwrap();
        }
        writeln!(f).unwrap();
    }
}
 
pub fn save_blocking_curve(config: &Config, output_dir: &str, x_y1: &[(f64, f64)], x_y2: &[(f64, f64)]) {
    let target_y = 1e-3;
    let x_for_y1 = find_x_for_y(x_y1, target_y);
    let x_for_y2 = find_x_for_y(x_y2, target_y);
    
    let mut f = get_mut_file(&format!("{output_dir}/blocking_curve.txt"));
    for ((x, y1), (_x, y2)) in x_y1.iter().zip(x_y2.iter()) {
        writeln!(f, "{:.2} {:.5} {:.5}", x, y1, y2).unwrap();
    }
    
    let mut f = get_mut_file(&format!("{output_dir}/network_capacity.txt"));
    if let Some(x_for_y1) = x_for_y1 {
        writeln!(f, "WXC-based NW: {:.5}", x_for_y1).unwrap();
    } else {
        writeln!(f, "WXC-based NW: NOT MEASURED").unwrap();
    }
    if let Some(x_for_y2) = x_for_y2 {
        writeln!(f, "Layer NW: {:.5}", x_for_y2).unwrap();
    } else {
        writeln!(f, "Layer NW: NOT MEASURED").unwrap();
    }
    match x_for_y1.is_some() && x_for_y2.is_some() {
        true => {
            let degradation_percent = (1.0 - x_for_y2.unwrap() / x_for_y1.unwrap()) * 100.0;
            writeln!(f, "Degradation: {:.2}%", degradation_percent).unwrap();
        }
        false => writeln!(f, "Degradation NOT MEASURED").unwrap(),
    }
 
    let mut child = std::process::Command::new(config.simulation.pythonexe_path.clone())
            .arg(CURVE_GRAPH_SCRIPT)
            .args(vec![format!("{}/blocking_curve.txt", config.simulation.outdir), format!("{}/network_capacity.txt", config.simulation.outdir), format!("{}/blocking_curve.png", config.simulation.outdir)])
            .spawn()
            .expect("something error");
    child.wait().unwrap();
}
 
fn save_wxc_port_pass_count_img(config: &Config) {
    // println!("{} {} {}", config.simulation.pythonexe_path, format!("{}/conv/wxc_port_pass_count.txt", config.simulation.outdir), format!("{}/prop/wxc_port_pass_count.txt", config.simulation.outdir));
    let mut child = std::process::Command::new(config.simulation.pythonexe_path.clone())
            .arg(TRAVERSE_GRAPH_SCRIPT)
            .args(vec![
                format!("{}/conv/wxc_port_pass_count.txt", config.simulation.outdir),
                format!("{}/prop/wxc_port_pass_count.txt", config.simulation.outdir),
                format!("{}/wxc_port_pass_count.png", config.simulation.outdir)])
            .spawn()
            .expect("something error");
    child.wait().unwrap();
}
 
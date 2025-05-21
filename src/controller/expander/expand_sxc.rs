use fxhash::FxHashMap;

use crate::{config::Config, debugger, network::{CoreIndex, FiberID, Network, PortID, XCType}, np_core::parameters::CORE_FACTOR, topology::RouteCandidate, Edge};

use super::generate_new_mc_fiber;

pub fn expand_sxc_fibers(config: &Config, network: &mut Network, target_edges: &[Edge]) {

    if target_edges.len() < 2 {
        panic!("`target_edges` should be longer than two");
    }

    // 区間上にファイバを増設するべきか判断する
    // 既に存在している場合は増設せず，コアを選択して配置すること
    // コア番号ごとに探索し，検索する
    let mut fiber_sequences: FxHashMap<CoreIndex, Vec<Option<FiberID>>> = FxHashMap::default();

    for core_index_as_usize in 0..CORE_FACTOR {
        let core_index = CoreIndex::new(core_index_as_usize);

        let entry = fiber_sequences.entry(core_index).or_default(); // same as vec![]
        
        // 最初のファイバを探す
        {
            let first_edge = target_edges.first().unwrap();
            let fiber_ids_on_first_edge = network.get_fiber_id_on_edge(first_edge);
            let first_fiber_id = get_fiber_contains_unused_core_specified(network, &fiber_ids_on_first_edge, &core_index, XCType::Wxc, XCType::Sxc);
            entry.push(first_fiber_id);
        }
        
        // 中間のファイバを探す
        for (index, intermediate_edge) in target_edges.iter().enumerate() {
            if index == 0 || index == target_edges.len() - 1 {
                continue;
            }

            let fiber_ids_on_intermediate_edge = network.get_fiber_id_on_edge(intermediate_edge);
            let intermediate_fiber_id = get_fiber_contains_unused_core_specified(network, &fiber_ids_on_intermediate_edge, &core_index, XCType::Sxc, XCType::Sxc);
            entry.push(intermediate_fiber_id);
        }
        
        // 最後のファイバを探す
        {
            let last_edge = target_edges.last().unwrap();
            let fiber_ids_on_last_edge = network.get_fiber_id_on_edge(last_edge);
            let last_fiber_id = get_fiber_contains_unused_core_specified(network, &fiber_ids_on_last_edge, &core_index, XCType::Sxc, XCType::Wxc);
            entry.push(last_fiber_id);
        }

        // ここまでにNoneがないとき，この候補で対応可能
        // これ以上の探索は無用
        if !entry.contains(&None) {
            break;
        }
    }

    // ファイバ増設回数を最小にする組み合わせ(CoreIndex)を抽出したい
    let mut core_index_fiber_seq_vec: Vec<(CoreIndex, Vec<Option<FiberID>>)> = fiber_sequences.into_iter().collect();
    core_index_fiber_seq_vec.sort_by_key(|(a_core_index, _)| a_core_index.index());
    core_index_fiber_seq_vec.sort_by_key(|(_a_core_index, a_fiber_seq)| a_fiber_seq.iter().filter(|x| x.is_none()).count());

    // 上記結果に合わせて，必要なファイバを敷設
    let (core_index, fiber_seq) = core_index_fiber_seq_vec.first().unwrap();
    let core_index_as_usize = core_index.index();

    // 最初のファイバを設立 or 取得
    #[allow(unused_assignments)]
    let mut prev_dst_port = PortID::nil();
    {
        let first_fiber = match fiber_seq.first().unwrap() {
            Some(first_fiber_id) => network.get_fiber_by_id(first_fiber_id),
            None => {
                let first_fiber = generate_new_mc_fiber(network, target_edges.first().unwrap(), XCType::Wxc, XCType::Sxc);
                let fibers = vec![first_fiber];
                debugger::log_fibers_expand(config, network, &fibers);
                network.regist_fiber(fibers[0].clone())
            },
        };
        prev_dst_port = first_fiber.dst_port_ids[core_index_as_usize];
        first_fiber.fiber_id
    };

    // 中間のファイバを設立 or 取得
    for (index, (intermediate_fiber_id, target_edge)) in fiber_seq.iter().zip(target_edges.iter()).enumerate() {
        if index == 0 || index == target_edges.len() - 1 {
            continue;
        }

        let intermediate_fiber = match intermediate_fiber_id {
            Some(intermediate_fiber_id) => network.get_fiber_by_id(intermediate_fiber_id),
            None => {
                let intermediate_fiber = generate_new_mc_fiber(network, target_edge, XCType::Sxc, XCType::Sxc);
                let fibers = vec![intermediate_fiber];
                debugger::log_fibers_expand(config, network, &fibers);
                network.regist_fiber(fibers[0].clone())
            },
        };
        let intermediate_fiber_src_port = intermediate_fiber.src_port_ids[core_index_as_usize];
        let intermediate_fiber_dst_port = intermediate_fiber.dst_port_ids[core_index_as_usize];

        let xc = network.get_xc_mut_on_node(target_edge.src.into(), &XCType::Sxc);
        xc.connect_io(&prev_dst_port, &intermediate_fiber_src_port).unwrap_or_else(|err| {
            println!("{err}");
            panic!();
        });

        prev_dst_port = intermediate_fiber_dst_port;
    }

    // 最後のファイバを設立
    {
        let last_fiber = match fiber_seq.last().unwrap() {
            Some(last_fiber_id) => network.get_fiber_by_id(last_fiber_id),
            None => {
                let last_fiber = generate_new_mc_fiber(network, target_edges.last().unwrap(), XCType::Sxc, XCType::Wxc);
                let fibers = vec![last_fiber];
                debugger::log_fibers_expand(config, network, &fibers);
                network.regist_fiber(fibers[0].clone())
            },
        };
        let last_fiber_src_port = last_fiber.src_port_ids[core_index_as_usize];

        let xc = network.get_xc_mut_on_node(target_edges.last().unwrap().src.into(), &XCType::Sxc);
        xc.connect_io(&prev_dst_port, &last_fiber_src_port).unwrap_or_else(|err| {
            println!("{err}");
            panic!();
        });
    }

    // ログ
    debugger::log_core_bypass(config, target_edges, core_index);
}

fn get_fiber_contains_unused_core_specified(network: &Network, fiber_ids_on_edge: &[FiberID], core_index: &CoreIndex, src_type: XCType, dst_type: XCType) -> Option<FiberID> {

    for fiber_id in fiber_ids_on_edge {
        let fiber = network.get_fiber_by_id(fiber_id);
        let [target_src_xc_type, target_dst_xc_type] = network.get_fiber_sd_xc_type(fiber);
        if target_src_xc_type == src_type && target_dst_xc_type == dst_type {
            // コアに空きがあるか?
            let unused_core = network.get_unused_core(fiber);
            if unused_core.contains(core_index) {
                return Some(*fiber_id);
            }
        }
    }

    None
}

pub fn get_min_expand_route_cand(network: &Network, route_cands: &[&RouteCandidate]) -> RouteCandidate {

    let mut expand_times = vec![];

    for route_cand in route_cands {
        let mut min_expand_count = 0;

        for core_index_as_usize in 0..CORE_FACTOR {
            let core_index = CoreIndex::new(core_index_as_usize);

            let mut expand_count = 0;
            for edge in &route_cand.edge_route {
                let fiber_ids_on_edge = network.get_fiber_id_on_edge(edge);
                if get_fiber_contains_unused_core_specified(network, &fiber_ids_on_edge, &core_index, XCType::Wxc, XCType::Sxc).is_none() {
                    expand_count += 1;
                }
            }
            if expand_count < min_expand_count {
                min_expand_count = expand_count;
            }
        }

        expand_times.push(min_expand_count);
    }

    let (index, _count) = expand_times.iter().enumerate().min_by_key(|(idx, _count)| *idx).unwrap();

    route_cands[index].clone()
}
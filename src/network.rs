use std::{collections::HashMap, fmt::Display, hash::BuildHasherDefault};

use crate::{
    controller::expander::generate_new_fiber,
    np_core::{
        parameters::{CORE_FACTOR, SLOT, WAVEBAND_COUNT},
        StateMatrix,
    },
    topology::RouteCandidate,
    utils::contains_subslice,
    Node, WBIndex, SD,
};
use fxhash::{FxBuildHasher, FxHashMap, FxHashSet};
use layer_to_np2::debug_println;
use layer_top::LayerTopology;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use strum::IntoEnumIterator;
pub use xc::PortID;
use xc::XCID;

use crate::{config::Config, debugger, demand::Demand, topology::Topology, Edge};

pub mod nw_utils;
pub mod state_matrix;

mod layer_top;

const UPDATE_LAYER_TOPOLOGIES: bool = true;

#[derive(Debug, Clone)]
pub struct Network {
    fibers: FxHashMap<FiberID, Fiber>,
    fiber_ids_on_edges: FxHashMap<Edge, Vec<FiberID>>,
    pub edges: Vec<Edge>,
    pub xcs: FxHashMap<XCID, XC>,
    edge_costs: FxHashMap<Edge, f64>,
    empty_fiber_ids_on_edges_cache: FxHashMap<Edge, Vec<FiberID>>,
    pub rng: ChaCha8Rng,
    portid_to_xcid: FxHashMap<PortID, XCID>,
    layer_topologies: FxHashMap<XCType, LayerTopology>,
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fibers_string = String::new();
        self.fibers.iter().for_each(|(_, fiber)| {
            fibers_string += &format!(
                "Edge: {}, Type: {}-{}\n",
                fiber.edge, fiber.sd_xc_type[0], fiber.sd_xc_type[1]
            );
        });

        write!(f, "fibers: {fibers_string}")
    }
}

impl Network {
    pub fn new(config: &Config, topology: &Topology, xc_types: &[XCType]) -> Self {
        // Fibers
        let fibers: FxHashMap<FiberID, Fiber> = FxHashMap::default();
        let fiber_ids_on_edges: FxHashMap<Edge, Vec<FiberID>> = FxHashMap::default();

        // For What?
        let edges: Vec<Edge> = topology.edges.clone();

        // XCs
        let mut xcs: FxHashMap<XCID, XC> = FxHashMap::default();
        let node_count: usize = topology.link_matrix.len();
        for node in 0..node_count {
            for xc_type in xc_types {
                let xc: XC = XC::new(node, *xc_type);
                xcs.insert(xc.id, xc);
            }
        }

        // For chache
        let edge_costs: FxHashMap<Edge, f64> = FxHashMap::default();
        let empty_fiber_ids_on_edges_cache: FxHashMap<Edge, Vec<FiberID>> = FxHashMap::default();
        let portid_to_xcid = FxHashMap::default();

        // For new Routerrrr
        let layer_topologies = FxHashMap::default();

        // For randomize
        let rng: ChaCha8Rng = ChaCha8Rng::seed_from_u64(config.simulation.random_seed);

        let mut network = Network {
            fibers,
            fiber_ids_on_edges,
            edges,
            xcs,
            edge_costs,
            empty_fiber_ids_on_edges_cache,
            rng,
            portid_to_xcid,
            layer_topologies,
        };

        for &edge in &topology.edges {
            let fiber = generate_new_fiber(&mut network, &edge, XCType::Wxc, XCType::Wxc);
            network
                .fiber_ids_on_edges
                .insert(edge, vec![fiber.fiber_id]);
            network
                .empty_fiber_ids_on_edges_cache
                .insert(edge, vec![fiber.fiber_id]);

            network.fibers.insert(fiber.fiber_id, fiber);
            network.calc_edge_cost(&edge);
        }

        network
    }

    pub fn get_all_fibers(&self) -> &FxHashMap<FiberID, Fiber> {
        &self.fibers
    }

    /// 指定されたエッジ上のすべてのファイバを返す
    pub fn get_fibers_on_edge(&self, edge: &Edge) -> Vec<&Fiber> {
        self.fibers
            .values()
            .filter(|fiber| &fiber.edge == edge)
            .collect()
    }

    /// 指定されたエッジ上に指定されたXCTypeを持つファイバが存在するかを返す
    pub fn has_fiber_on_edge_with_xc_types(
        &self,
        edge: &Edge,
        src_type: XCType,
        dst_type: XCType,
    ) -> bool {
        self.fibers.values().any(|fiber| {
            &fiber.edge == edge
                && fiber.sd_xc_type[0] == src_type
                && fiber.sd_xc_type[1] == dst_type
        })
    }

    pub fn get_all_edges(&self) -> Vec<Edge> {
        let mut edge_set: FxHashSet<Edge> = FxHashSet::default();

        for fiber in self.fibers.values() {
            edge_set.insert(fiber.edge.clone());
        }

        edge_set.into_iter().collect()
    }

    pub fn get_nodes(&self) -> Vec<Node> {
        let mut output = vec![];

        for edge in &self.edges {
            if !output.contains(&edge.src) {
                output.push(edge.src);
            }
            if !output.contains(&edge.dst) {
                output.push(edge.dst);
            }
        }

        output
    }

    pub fn get_fiber_breakdown(&self) -> FxHashMap<[XCType; 2], usize> {
        let mut counter = FxHashMap::default();

        for fiber in self.get_fibers().values() {
            let entry = counter.entry(fiber.sd_xc_type).or_insert(0);
            *entry += 1;
        }

        counter
    }

    pub fn get_fiber_sd_xc_type_by_id(&self, fiber_id: &FiberID) -> [XCType; 2] {
        let fiber = self.get_fiber_by_id(fiber_id);
        self.get_fiber_sd_xc_type(fiber)
    }

    pub fn get_fiber_sd_xc_type(&self, fiber: &Fiber) -> [XCType; 2] {
        fiber.sd_xc_type
    }

    /// edge上に存在するfiberをすべて返す
    pub fn get_fiber_id_on_edge(&self, edge: &Edge) -> Vec<FiberID> {
        match self.fiber_ids_on_edges.get(edge) {
            Some(fiber_id_on_edge) => fiber_id_on_edge.clone(),
            None => vec![],
        }
    }

    pub fn get_fiber_ids_on_edge_empty(&self, edge: &Edge) -> Vec<FiberID> {
        self.empty_fiber_ids_on_edges_cache
            .get(edge)
            .unwrap()
            .to_vec()
    }

    /// edge上に存在するfiberをすべて返す
    /// なお，WXC2WXCについては，複数のファイバを横断し，
    /// すべての波長スロットが使用可能になった段階で他のファイバ出力を止める
    pub fn get_fiber_id_on_edge_partial(&self, edge: &Edge) -> Vec<FiberID> {
        match self.empty_fiber_ids_on_edges_cache.get(edge) {
            // match self.fiber_id_on_edges.get(edge) {
            Some(fiber_id_on_edge) => {
                let mut target_state_matrix = StateMatrix::new_fulfilled();
                let mut empty_flag = false;

                let mut out = vec![];

                for fiber_id in fiber_id_on_edge {
                    let fiber = self.get_fiber_by_id(fiber_id);
                    if fiber.sd_xc_type == [XCType::Wxc, XCType::Wxc] {
                        if empty_flag {
                            continue;
                        }

                        if target_state_matrix.is_empty() {
                            empty_flag = true;
                            continue;
                        }

                        let prev_target_state_matrix = target_state_matrix;
                        target_state_matrix &= fiber.state_matrixes[0]; // Because, Wxc to Wxc should be SMF

                        if target_state_matrix != prev_target_state_matrix {
                            out.push(*fiber_id);
                        }
                    } else {
                        out.push(*fiber_id);
                    }
                }

                out
            }
            None => vec![],
        }
    }

    pub fn get_edge_cost(&self) -> &FxHashMap<Edge, f64> {
        &self.edge_costs
    }

    /// fibersを返す
    pub fn get_fibers(&self) -> &FxHashMap<FiberID, Fiber> {
        &self.fibers
    }

    pub fn get_fiber_sequence_core(
        &self,
        first_fiber: &Fiber,
        core_index: &CoreIndex,
    ) -> Option<Vec<FiberID>> {
        let mut fiber_seq = vec![first_fiber.fiber_id];

        let dst_xc = self.get_xc_by_input_port_id(&first_fiber.dst_port_ids[core_index.index()]);
        let next_src_port = dst_xc.get_route(&first_fiber.dst_port_ids[core_index.index()])?;
        let mut target_fiber = self.get_fiber_by_src_device(&next_src_port);
        // debug_println!(dst_xc);

        loop {
            fiber_seq.push(target_fiber.fiber_id);

            if self.get_fiber_sd_xc_type(target_fiber)[1] == XCType::Wxc {
                // バイパス終端
                break;
            }

            let dst_xc =
                self.get_xc_by_input_port_id(&target_fiber.dst_port_ids[core_index.index()]);
            if let Some(next_src_port) =
                dst_xc.get_route(&target_fiber.dst_port_ids[core_index.index()])
            {
                target_fiber = self.get_fiber_by_src_device(&next_src_port);
            } else {
                return None;
            }
        }

        Some(fiber_seq)
    }

    pub fn delete_empty_fibers_core(&mut self, config: &Config, taboo_list: &mut Vec<SD>) {
        self.delete_empty_fibers(config, taboo_list);
    }

    pub fn delete_empty_fibers_wb(&mut self, config: &Config, taboo_list: &mut Vec<SD>) {
        let mut delete_bypasses: Vec<(WBIndex, Vec<FiberID>)> = vec![];

        for first_fiber in self.fibers.values() {
            // WBバイパスに対して処理を実施
            // WBバイパス区間の収容率を見る
            // WXC2WBXCファイバを発見次第，検索を開始
            // あるWBに対して(そのファイバでバイパスが始まっているものに対して)収容率を計算
            // 収容率がゼロなら，削除対象に追加する

            let fiber_type = self.get_fiber_sd_xc_type(first_fiber);
            if fiber_type == [XCType::Wxc, XCType::Wbxc] {
                let xc = self.get_xc_by_input_port_id(&first_fiber.dst_port_ids[0]);
                'wb_loop: for wb in WBIndex::iter() {
                    if xc
                        .get_route_wbxc_wb(&first_fiber.dst_port_ids[0], wb)
                        .is_some()
                    {
                        // バイパスの先頭!
                        // 現在のxcに対して，WBの使用状況を照会
                        // 埋まっていればOK，埋まっていなければNG
                        // バイパス区間は途中でAdd/Dropできないので，
                        // 最初のファイバさえ見られれば，Ok/NGが決まる
                        // しかし，バイパスの削除には全区間のファイバのIDが必要なので，
                        // 判定後に削除する

                        // println!("{} {:?} {} {:?}", first_fiber.edge, first_fiber.sd_xc_type, first_fiber.state_matrixes[0], self.get_fiber_sequence_wb(first_fiber, &wb));

                        for i in 0..SLOT / WAVEBAND_COUNT {
                            if first_fiber.state_matrixes[0][wb.index() * SLOT / WAVEBAND_COUNT + i]
                            {
                                // 埋まっているので，使用されている!
                                continue 'wb_loop;
                            }
                        }

                        // WB内すべてのスロットが空いていた
                        // バイパスを取得する
                        if let Some(fiber_seq) = self.get_fiber_sequence_wb(first_fiber, &wb) {
                            delete_bypasses.push((wb, fiber_seq));
                        }
                    }
                    // バイパスの先頭でなければ用はない
                }
            }
        }

        for (wb, fiber_seq) in delete_bypasses.iter() {
            // バイパスファイバに接続されているXCからwbに関するテーブルを削除するように，tmpに追加する
            // 初めのファイバを飛ばして，二番目から探索，src_xcだけを確認すればよい
            let first_fiber = self.get_fiber_by_id(fiber_seq.first().unwrap());
            let mut prev_input_device_id = first_fiber.dst_port_ids.clone();

            for fiber_id in fiber_seq.iter().skip(1) {
                let fiber = self.get_fiber_by_id(fiber_id);
                let output_device_id = fiber.src_port_ids.clone();

                let xc = self.get_xc_mut_by_output_port_id(&output_device_id[0]);
                xc.disconnect_io_wb(&prev_input_device_id[0], &output_device_id[0], wb)
                    .unwrap_or_else(|err| {
                        eprintln!("{err}");
                        panic!();
                    });

                let fiber = self.get_fiber_by_id(fiber_id);
                prev_input_device_id = fiber.dst_port_ids.clone();
            }

            let first_fiber = self.get_fiber_by_id(fiber_seq.first().unwrap());
            let last_fiber = self.get_fiber_by_id(fiber_seq.last().unwrap());
            let sd = SD::new_from_nodes(first_fiber.edge.src, last_fiber.edge.dst);
            taboo_list.push(sd);
            debugger::log_taboo_list_addition(config, &sd);
            debugger::log_wb_bypass_remove(
                config,
                &fiber_seq
                    .iter()
                    .map(|x| self.get_fiber_by_id(x).edge)
                    .collect::<Vec<_>>(),
                wb,
            );
        }

        self.delete_empty_fibers(config, taboo_list);
    }

    pub fn get_fiber_sequence_wb(&self, first_fiber: &Fiber, wb: &WBIndex) -> Option<Vec<FiberID>> {
        // GET FIBER SEQUENCE
        let mut fiber_seq = vec![first_fiber.fiber_id];

        let dst_xc = self.get_xc_by_input_port_id(&first_fiber.dst_port_ids[0]);
        let src_device = dst_xc.get_route_wbxc_wb(&first_fiber.dst_port_ids[0], *wb)?;
        let mut target_fiber = self.get_fiber_by_src_device(&src_device);

        loop {
            fiber_seq.push(target_fiber.fiber_id);

            if self.get_fiber_sd_xc_type(target_fiber)[1] == XCType::Wxc {
                break;
            }

            let dst_xc = self.get_xc_by_input_port_id(&target_fiber.dst_port_ids[0]);
            let src_device = dst_xc.get_route_wbxc_wb(&target_fiber.dst_port_ids[0], *wb)?;
            target_fiber = self.get_fiber_by_src_device(&src_device);
        }

        Some(fiber_seq)
    }

    pub fn delete_empty_fibers(&mut self, config: &Config, taboo_list: &mut Vec<SD>) {
        let mut delete_fiber_ids: Vec<FiberID> = vec![];
        let mut delete_fiber_edges = vec![];
        for (_fiber_id, fiber) in self.fibers.iter() {
            // ファイバの削除
            if fiber.occupancy == 0 {
                delete_fiber_ids.push(fiber.fiber_id);
                delete_fiber_edges.push(fiber.edge);

                // TABOO_LISTへの追加
                let fiber_type = self.get_fiber_sd_xc_type(fiber);
                if fiber_type == [XCType::Wxc, XCType::Fxc] {
                    let seq = self.get_fiber_sequence_as_edges(fiber).unwrap();
                    let sd = SD::new(
                        seq.first().unwrap().src.into(),
                        seq.last().unwrap().dst.into(),
                    );
                    debugger::log_taboo_list_addition(config, &sd);
                    taboo_list.push(sd);
                    debugger::log_core_bypass_remove(config, &seq, &CoreIndex::new(0));
                }
            }
        }

        for delete_fiber_id in &delete_fiber_ids {
            self.delete_fiber(config, delete_fiber_id);
        }
    }

    /// IDで指定してファイバを削除する
    pub fn delete_fiber(&mut self, config: &Config, fiber_id: &FiberID) {
        let fiber_sd_xc_type = self.get_fiber_sd_xc_type_by_id(fiber_id);
        let fiber = self.get_fiber_mut_by_id(fiber_id);

        debugger::log_fiber_remove(config, fiber);
        let edge = fiber.edge;
        let src_device_id = fiber.src_port_ids.clone();
        let dst_device_id = fiber.dst_port_ids.clone();

        self.fibers.remove(fiber_id);

        if fiber_sd_xc_type.contains(&XCType::Sxc) {
            for core_index_as_usize in 0..CORE_FACTOR {
                let src_xc = self.get_xc_mut_by_output_port_id(&src_device_id[core_index_as_usize]);
                src_xc.remove_device(src_device_id[core_index_as_usize], false);
                let dst_xc = self.get_xc_mut_by_input_device(&dst_device_id[core_index_as_usize]);
                dst_xc.remove_device(dst_device_id[core_index_as_usize], true);
            }
        } else {
            let src_xc = self.get_xc_mut_by_output_port_id(&src_device_id[0]);
            src_xc.remove_device(src_device_id[0], false);
            let dst_xc = self.get_xc_mut_by_input_device(&dst_device_id[0]);
            dst_xc.remove_device(dst_device_id[0], true);
        }

        // cache_update
        self.empty_fiber_ids_on_edges_cache
            .get_mut(&edge)
            .unwrap()
            .retain(|&x| x != *fiber_id);
        self.fiber_ids_on_edges
            .get_mut(&edge)
            .unwrap()
            .retain(|&x| x != *fiber_id);
        self.calc_edge_cost(&edge);
    }

    /// IDで指定してファイバ(参照)を取得する
    pub fn get_fiber_by_id(&self, fiber_id: &FiberID) -> &Fiber {
        match self.fibers.get(fiber_id) {
            Some(fiber) => fiber,
            None => panic!("The fiber is not exist."),
        }
    }

    /// IDで指定してファイバ(参照)を取得する
    pub fn get_fiber_mut_by_id(&mut self, fiber_id: &FiberID) -> &mut Fiber {
        match self.fibers.get_mut(fiber_id) {
            Some(fiber) => fiber,
            None => panic!("The fiber is not exist."),
        }
    }

    pub fn calc_edge_cost(&mut self, edge: &Edge) {
        let fiber_id_on_edge = &self.get_fiber_id_on_edge(edge);
        let mut residual = 0;
        let mut capacity = 0;
        for fiber_id in fiber_id_on_edge {
            let fiber = self.get_fiber_by_id(fiber_id);
            residual += fiber.residual;
            capacity += fiber.residual + fiber.occupancy;
        }
        self.edge_costs
            .insert(*edge, (capacity as f64) / ((residual as f64) + 0.01));
    }

    /// slot, target_fibersを指定してNetworkにDemandを割り当てる
    pub fn assign_path(
        &mut self,
        slots: Vec<usize>,
        target_fiber_ids: &[FiberID],
        core_indices: &[CoreIndex],
        demand: &Demand,
    ) {
        self.assign_path_da(slots, 1, target_fiber_ids, core_indices, demand);
    }

    pub fn assign_path_da(
        &mut self,
        slots: Vec<usize>,
        width: usize,
        target_fiber_ids: &[FiberID],
        core_indices: &[CoreIndex],
        demand: &Demand,
    ) {
        for target_fiber_id in target_fiber_ids.iter() {
            let target_fiber = self.get_fiber_by_id(target_fiber_id);
            if target_fiber.sd_xc_type == [XCType::Wxc, XCType::Wbxc] {
                let fiber_seq = self
                    .get_fiber_sequence_wb(target_fiber, &WBIndex::from_wavelength(slots[0]))
                    .unwrap();
                if !contains_subslice(target_fiber_ids, &fiber_seq) {
                    debug_println!(target_fiber_ids);
                    debug_println!(fiber_seq);

                    debug_println!(target_fiber_ids
                        .iter()
                        .map(|x| self.get_fiber_by_id(x).edge)
                        .collect::<Vec<_>>());
                    debug_println!(fiber_seq
                        .iter()
                        .map(|x| self.get_fiber_by_id(x).edge)
                        .collect::<Vec<_>>());

                    panic!();
                }
            }
        }

        for (target_fiber_id, core_index) in target_fiber_ids.iter().zip(core_indices.iter()) {
            let fiber = self.get_fiber_mut_by_id(target_fiber_id);
            let edge = fiber.edge;
            fiber.assign(slots[0], width, core_index, demand.index);

            if fiber.is_full() {
                self.empty_fiber_ids_on_edges_cache
                    .get_mut(&edge)
                    .unwrap()
                    .retain(|&x| x != *target_fiber_id);
            }

            self.calc_edge_cost(&edge);
        }
    }

    /// 登録済みのDemandをNetworkから削除する
    pub fn remove_path(&mut self, demand: &Demand) {
        self.remove_path_da(demand);
    }

    /// 登録済みのDemandをNetworkから削除する
    pub fn remove_path_da(&mut self, demand: &Demand) {
        for (target_fiber_id, core_index) in demand.fiber_ids.iter().zip(demand.core_indices.iter())
        {
            let fiber = self.get_fiber_mut_by_id(target_fiber_id);
            let edge = fiber.edge;

            // cache_update
            let fiber_is_full = fiber.is_full();
            fiber.delete(
                demand.slot_heads[0],
                demand.slot_width,
                core_index,
                demand.index,
            );

            if fiber_is_full {
                self.empty_fiber_ids_on_edges_cache
                    .get_mut(&edge)
                    .unwrap()
                    .push(*target_fiber_id);
            }

            // calc edge costs
            self.calc_edge_cost(&edge);
        }
    }

    pub fn get_xc_on_node(&self, node: usize, xc_type: &XCType) -> Option<&XC> {
        if let Some((_, xc)) = self
            .xcs
            .iter()
            .find(|(_, p)| p.node == node && p.xc_type == *xc_type)
        {
            Some(xc)
        } else {
            None
        }
    }

    pub fn get_xc_mut_on_node(&mut self, node: usize, xc_type: &XCType) -> &mut XC {
        if self
            .xcs
            .iter()
            .any(|(_, p)| p.node == node && p.xc_type == *xc_type)
        {
            return self
                .xcs
                .iter_mut()
                .find(|(_, p)| p.node == node && p.xc_type == *xc_type)
                .unwrap()
                .1;
        }

        let xc = XC::new(node, *xc_type);
        self.xcs.entry(xc.id).or_insert(xc)
    }

    pub fn get_xc_by_input_port_id(&self, input_device_id: &PortID) -> &XC {
        // let (_, xc) = self.xcs
        //     .iter()
        //     .find(|(_, p)| p.has_input_device(input_device_id))
        //     .unwrap();
        // xc

        let xcid = self.portid_to_xcid.get(input_device_id).unwrap();
        let xc = self.xcs.get(xcid).unwrap();

        if !xc.has_input_device(input_device_id) {
            panic!();
        }

        xc
    }

    pub fn get_xc_mut_by_input_device(&mut self, input_device_id: &PortID) -> &mut XC {
        // let (_, xc) = self.xcs
        //     .iter_mut()
        //     .find(|(_, p)| p.has_input_device(input_device_id))
        //     .unwrap();
        // xc

        let xcid = self.portid_to_xcid.get(input_device_id).unwrap();
        let xc = self.xcs.get_mut(xcid).unwrap();

        if !xc.has_input_device(input_device_id) {
            panic!();
        }

        xc
    }

    pub fn get_xc_by_output_port_id(&self, output_device_id: &PortID) -> &XC {
        // let (_, xc) = self.xcs
        //     .iter()
        //     .find(|(_, p)| p.has_output_device(output_device_id))
        //     .unwrap();
        // xc

        let xcid = self.portid_to_xcid.get(output_device_id).unwrap();
        let xc = self.xcs.get(xcid).unwrap();

        if !xc.has_output_device(output_device_id) {
            panic!();
        }

        xc
    }

    pub fn get_xc_mut_by_output_port_id(&mut self, output_device_id: &PortID) -> &mut XC {
        // let (_, xc) = self.xcs
        //     .iter_mut()
        //     .find(|(_, p)| p.has_output_device(output_device_id))
        //     .unwrap();
        // xc

        let xcid = self.portid_to_xcid.get(output_device_id).unwrap();
        let xc = self.xcs.get_mut(xcid).unwrap();

        if !xc.has_output_device(output_device_id) {
            panic!();
        }

        xc
    }

    pub fn get_xc_by_io_device(&self, input_device_id: &PortID, output_device_id: &PortID) -> &XC {
        let (_, xc) = self
            .xcs
            .iter()
            .find(|(_, p)| p.has_output_device(output_device_id))
            .unwrap();

        if xc.has_input_device(input_device_id) {
            xc
        } else {
            panic!();
        }
    }

    pub fn regist_fibers(&mut self, fibers: Vec<Fiber>) {
        for fiber in &fibers {
            if !self.edges.contains(&fiber.edge) {
                panic!("The network has no edge: {}", fiber.edge);
            }

            // fiber_id_on_edges 関連
            if let Some(fiber_id_on_edge) = self.fiber_ids_on_edges.get_mut(&fiber.edge) {
                fiber_id_on_edge.push(fiber.fiber_id);
            } else {
                self.fiber_ids_on_edges
                    .insert(fiber.edge, vec![fiber.fiber_id]);
            }
        }

        // Cache 関連
        for fiber in fibers {
            let edge = fiber.edge;
            // cache_update
            self.empty_fiber_ids_on_edges_cache
                .get_mut(&edge)
                .unwrap()
                .push(fiber.fiber_id);

            self.fibers.insert(fiber.fiber_id, fiber);
            self.calc_edge_cost(&edge);
        }
    }

    pub fn regist_fiber(&mut self, fiber: Fiber) -> &Fiber {
        if !self.edges.contains(&fiber.edge) {
            panic!("The network has no edge: {}", fiber.edge);
        }

        let fiber_id = fiber.fiber_id;

        // fiber_id_on_edges 関連
        if let Some(fiber_id_on_edge) = self.fiber_ids_on_edges.get_mut(&fiber.edge) {
            fiber_id_on_edge.push(fiber.fiber_id);
        } else {
            self.fiber_ids_on_edges
                .insert(fiber.edge, vec![fiber.fiber_id]);
        }

        let edge = fiber.edge;
        // cache_update
        self.empty_fiber_ids_on_edges_cache
            .get_mut(&edge)
            .unwrap()
            .push(fiber.fiber_id);

        self.fibers.insert(fiber.fiber_id, fiber);
        self.calc_edge_cost(&edge);

        self.get_fiber_by_id(&fiber_id)
    }

    pub fn export(&self) -> FxHashMap<(EdgesType, Vec<Edge>), usize> {
        let mut o = FxHashMap::default();

        for fiber in self.fibers.values() {
            let [src_type, dst_type] = self.get_fiber_sd_xc_type(fiber);

            match src_type {
                XCType::Wxc => match dst_type {
                    XCType::Wxc => {
                        let entry = o.entry((EdgesType::Wxc, vec![fiber.edge])).or_insert(0);
                        *entry += 1;
                    }
                    XCType::Added_Wxc => {
                        let entry = o.entry((EdgesType::Wxc, vec![fiber.edge])).or_insert(0);
                        *entry += 1;
                    }
                    XCType::Wbxc => {
                        for wb in WBIndex::iter() {
                            if let Some(edge_seq) = self.get_fiber_sequence_as_edges_wb(fiber, &wb)
                            {
                                let entry = o.entry((EdgesType::Wbxc, edge_seq)).or_insert(0);
                                *entry += 1;
                            }
                        }
                    }
                    XCType::Fxc => {
                        let edge_seq = self.get_fiber_sequence_as_edges(fiber);
                        let entry = o.entry((EdgesType::Fxc, edge_seq.unwrap())).or_insert(0);
                        *entry += 1;
                    }
                    XCType::Sxc => {
                        for core_index_as_usize in 0..CORE_FACTOR {
                            if let Some(fiber_seq) = self.get_fiber_sequence_core(
                                fiber,
                                &CoreIndex::new(core_index_as_usize),
                            ) {
                                let edge_seq = fiber_seq
                                    .iter()
                                    .map(|fiber_id| self.get_fiber_by_id(fiber_id).edge)
                                    .collect();
                                let entry = o.entry((EdgesType::Sxc, edge_seq)).or_insert(0);
                                *entry += 1;
                            }
                        }
                    }
                },
                XCType::Added_Wxc => match dst_type {
                    XCType::Wxc => {
                        let entry = o.entry((EdgesType::Wxc, vec![fiber.edge])).or_insert(0);
                        *entry += 1;
                    }
                    XCType::Added_Wxc => {
                        let entry = o.entry((EdgesType::Wxc, vec![fiber.edge])).or_insert(0);
                        *entry += 1;
                    }
                    XCType::Wbxc => {
                        for wb in WBIndex::iter() {
                            if let Some(edge_seq) = self.get_fiber_sequence_as_edges_wb(fiber, &wb)
                            {
                                let entry = o.entry((EdgesType::Wbxc, edge_seq)).or_insert(0);
                                *entry += 1;
                            }
                        }
                    }
                    XCType::Fxc => {
                        let edge_seq = self.get_fiber_sequence_as_edges(fiber);
                        let entry = o.entry((EdgesType::Fxc, edge_seq.unwrap())).or_insert(0);
                        *entry += 1;
                    }
                    XCType::Sxc => {
                        for core_index_as_usize in 0..CORE_FACTOR {
                            if let Some(fiber_seq) = self.get_fiber_sequence_core(
                                fiber,
                                &CoreIndex::new(core_index_as_usize),
                            ) {
                                let edge_seq = fiber_seq
                                    .iter()
                                    .map(|fiber_id| self.get_fiber_by_id(fiber_id).edge)
                                    .collect();
                                let entry = o.entry((EdgesType::Sxc, edge_seq)).or_insert(0);
                                *entry += 1;
                            }
                        }
                    }
                },
                XCType::Wbxc | XCType::Fxc | XCType::Sxc => continue,
            }
        }

        o
    }

    pub fn get_fiber_by_src_device(&self, src_device: &PortID) -> &Fiber {
        self.fibers
            .iter()
            .find(|(_, x)| x.src_port_ids.contains(src_device))
            .unwrap_or_else(|| {
                debug_println!(self.get_xc_by_output_port_id(src_device));
                panic!();
            })
            .1
    }

    pub fn get_fiber_sequence(&self, first_fiber: &Fiber) -> Vec<FiberID> {
        // GET FIBER SEQUENCE
        let mut fiber_seq = vec![first_fiber.fiber_id];

        let dst_xc = self.get_xc_by_input_port_id(&first_fiber.dst_port_ids[0]);
        let src_device = dst_xc.get_route(&first_fiber.dst_port_ids[0]).unwrap();
        let mut target_fiber = self.get_fiber_by_src_device(&src_device);

        loop {
            fiber_seq.push(target_fiber.fiber_id);

            if self.get_fiber_sd_xc_type(target_fiber)[1] == XCType::Wxc {
                break;
            }

            let dst_xc = self.get_xc_by_input_port_id(&target_fiber.dst_port_ids[0]);
            let src_device = dst_xc.get_route(&target_fiber.dst_port_ids[0]).unwrap();
            target_fiber = self.get_fiber_by_src_device(&src_device);
        }

        fiber_seq
    }

    pub fn get_fiber_sequence_as_edges(&self, first_fiber: &Fiber) -> Option<Vec<Edge>> {
        match self.get_fiber_sd_xc_type(first_fiber) {
            [XCType::Wxc, XCType::Fxc] => {
                let fiber_seq = self.get_fiber_sequence(first_fiber);
                Some(
                    fiber_seq
                        .into_iter()
                        .map(|fiber_id| self.get_fiber_by_id(&fiber_id).edge)
                        .collect(),
                )
            }
            _ => unimplemented!(),
        }
    }

    pub fn get_fiber_sequence_as_edges_wb(
        &self,
        first_fiber: &Fiber,
        wb: &WBIndex,
    ) -> Option<Vec<Edge>> {
        match self.get_fiber_sd_xc_type(first_fiber) {
            [XCType::Wxc, XCType::Wbxc] => {
                self.get_fiber_sequence_wb(first_fiber, wb)
                    .map(|fiber_seq| {
                        fiber_seq
                            .into_iter()
                            .map(|fiber_id| self.get_fiber_by_id(&fiber_id).edge)
                            .collect()
                    })
            }
            _ => unimplemented!(),
        }
    }

    pub fn get_edges_advanced_double(&self) -> Vec<Vec<Edge>> {
        let wxc_edges = self.edges.clone();
        let mut fxc_edges = vec![];

        for fiber in self.fibers.values() {
            let [src_type, dst_type] = self.get_fiber_sd_xc_type(fiber);

            match src_type {
                XCType::Wxc => {
                    match dst_type {
                        XCType::Wxc => continue,
                        XCType::Added_Wxc => continue,
                        XCType::Wbxc => {
                            // GET FIBER SEQUENCE
                            for wb in WBIndex::iter() {
                                let mut edge_seq = vec![fiber.edge];

                                let dst_xc = self.get_xc_by_input_port_id(&fiber.dst_port_ids[0]);
                                if let Some(src_device) =
                                    dst_xc.get_route_wbxc_wb(&fiber.dst_port_ids[0], wb)
                                {
                                    let mut target_fiber =
                                        self.get_fiber_by_src_device(&src_device);

                                    loop {
                                        edge_seq.push(target_fiber.edge);

                                        if self.get_fiber_sd_xc_type(target_fiber)[1] == XCType::Wxc
                                        {
                                            fxc_edges.push(edge_seq);
                                            break;
                                        }

                                        let dst_xc = self
                                            .get_xc_by_input_port_id(&target_fiber.dst_port_ids[0]);
                                        let src_device = dst_xc
                                            .get_route_wbxc_wb(&target_fiber.dst_port_ids[0], wb)
                                            .unwrap();
                                        target_fiber = self.get_fiber_by_src_device(&src_device);
                                    }
                                }
                            }
                        }
                        XCType::Fxc => {
                            // GET FIBER SEQUENCE
                            let mut edge_seq = vec![fiber.edge];

                            let dst_xc = self.get_xc_by_input_port_id(&fiber.dst_port_ids[0]);
                            let src_device = dst_xc.get_route(&fiber.dst_port_ids[0]).unwrap();
                            let mut target_fiber = self.get_fiber_by_src_device(&src_device);

                            loop {
                                edge_seq.push(target_fiber.edge);

                                if self.get_fiber_sd_xc_type(target_fiber)[1] == XCType::Wxc {
                                    fxc_edges.push(edge_seq);
                                    break;
                                }

                                let dst_xc =
                                    self.get_xc_by_input_port_id(&target_fiber.dst_port_ids[0]);
                                let src_device =
                                    dst_xc.get_route(&target_fiber.dst_port_ids[0]).unwrap();
                                target_fiber = self.get_fiber_by_src_device(&src_device);
                            }
                        }
                        XCType::Sxc => {
                            for core_index_as_usize in 0..CORE_FACTOR {
                                if let Some(fiber_seq) = self.get_fiber_sequence_core(
                                    fiber,
                                    &CoreIndex::new(core_index_as_usize),
                                ) {
                                    let edge_seq = fiber_seq
                                        .iter()
                                        .map(|fiber_id| self.get_fiber_by_id(fiber_id).edge)
                                        .collect();
                                    fxc_edges.push(edge_seq);
                                }
                            }
                        }
                    }
                }
                XCType::Added_Wxc => {
                    match dst_type {
                        XCType::Wxc => continue,
                        XCType::Added_Wxc => continue,
                        XCType::Wbxc => {
                            // GET FIBER SEQUENCE
                            for wb in WBIndex::iter() {
                                let mut edge_seq = vec![fiber.edge];

                                let dst_xc = self.get_xc_by_input_port_id(&fiber.dst_port_ids[0]);
                                if let Some(src_device) =
                                    dst_xc.get_route_wbxc_wb(&fiber.dst_port_ids[0], wb)
                                {
                                    let mut target_fiber =
                                        self.get_fiber_by_src_device(&src_device);

                                    loop {
                                        edge_seq.push(target_fiber.edge);

                                        if self.get_fiber_sd_xc_type(target_fiber)[1] == XCType::Wxc
                                        {
                                            fxc_edges.push(edge_seq);
                                            break;
                                        }

                                        let dst_xc = self
                                            .get_xc_by_input_port_id(&target_fiber.dst_port_ids[0]);
                                        let src_device = dst_xc
                                            .get_route_wbxc_wb(&target_fiber.dst_port_ids[0], wb)
                                            .unwrap();
                                        target_fiber = self.get_fiber_by_src_device(&src_device);
                                    }
                                }
                            }
                        }
                        XCType::Fxc => {
                            // GET FIBER SEQUENCE
                            let mut edge_seq = vec![fiber.edge];

                            let dst_xc = self.get_xc_by_input_port_id(&fiber.dst_port_ids[0]);
                            let src_device = dst_xc.get_route(&fiber.dst_port_ids[0]).unwrap();
                            let mut target_fiber = self.get_fiber_by_src_device(&src_device);

                            loop {
                                edge_seq.push(target_fiber.edge);

                                if self.get_fiber_sd_xc_type(target_fiber)[1] == XCType::Wxc {
                                    fxc_edges.push(edge_seq);
                                    break;
                                }

                                let dst_xc =
                                    self.get_xc_by_input_port_id(&target_fiber.dst_port_ids[0]);
                                let src_device =
                                    dst_xc.get_route(&target_fiber.dst_port_ids[0]).unwrap();
                                target_fiber = self.get_fiber_by_src_device(&src_device);
                            }
                        }
                        XCType::Sxc => {
                            for core_index_as_usize in 0..CORE_FACTOR {
                                if let Some(fiber_seq) = self.get_fiber_sequence_core(
                                    fiber,
                                    &CoreIndex::new(core_index_as_usize),
                                ) {
                                    let edge_seq = fiber_seq
                                        .iter()
                                        .map(|fiber_id| self.get_fiber_by_id(fiber_id).edge)
                                        .collect();
                                    fxc_edges.push(edge_seq);
                                }
                            }
                        }
                    }
                }
                XCType::Wbxc => continue,
                XCType::Fxc => continue,
                XCType::Sxc => continue,
            }
        }

        let mut edges: Vec<Vec<Edge>> = wxc_edges.iter().map(|x| vec![*x]).collect();
        let fxc_edges_vec: Vec<Vec<Edge>> = fxc_edges.iter().map(|x| x.to_vec()).collect();
        edges.extend(fxc_edges_vec);

        edges
    }

    pub fn get_fiber_quality_distance_by_id(&self, fiber_id: &FiberID) -> usize {
        let [src_xc_type, dst_xc_type] = self.get_fiber_sd_xc_type_by_id(fiber_id);
        let fiber = self.get_fiber_by_id(fiber_id);

        xc_type_to_quality_distance(src_xc_type)
            + xc_type_to_quality_distance(dst_xc_type)
            + fiber.distance
    }

    /// To change the return value of this function to Range object, \
    /// You can implement core-search-order. \
    /// But this should be implemented by `router`.
    pub fn get_fiber_core_factor_by_id(&self, fiber_id: &FiberID) -> usize {
        let fiber = self.get_fiber_by_id(fiber_id);
        fiber.state_matrixes.len()
    }

    pub fn get_unused_core(&self, fiber: &Fiber) -> Vec<CoreIndex> {
        let [src_xc_type, dst_xc_type] = self.get_fiber_sd_xc_type(fiber);

        let mut used_cores = FxHashSet::default();

        match src_xc_type {
            XCType::Wxc | XCType::Wbxc | XCType::Fxc | XCType::Added_Wxc => (),
            XCType::Sxc => {
                let src_xc = self.get_xc_by_output_port_id(&fiber.src_port_ids[0]);
                for core_index_as_usize in 0..fiber.get_core_num() {
                    if src_xc.has_source(&fiber.src_port_ids[core_index_as_usize]) {
                        // Used
                        used_cores.insert(CoreIndex::new(core_index_as_usize));
                    } else {
                        // Unused
                    }
                }
            }
        }
        match dst_xc_type {
            XCType::Wxc | XCType::Wbxc | XCType::Fxc | XCType::Added_Wxc => (),
            XCType::Sxc => {
                let dst_xc = self.get_xc_by_input_port_id(&fiber.dst_port_ids[0]);
                for core_index_as_usize in 0..fiber.get_core_num() {
                    if dst_xc.has_destination(&fiber.dst_port_ids[core_index_as_usize]) {
                        // Used
                        used_cores.insert(CoreIndex::new(core_index_as_usize));
                    } else {
                        // Unused
                    }
                }
            }
        }

        let mut unused_cores = vec![];
        for core_index_as_usize in 0..fiber.get_core_num() {
            if !used_cores.contains(&CoreIndex::new(core_index_as_usize)) {
                unused_cores.push(CoreIndex::new(core_index_as_usize));
            }
        }

        unused_cores
    }

    pub fn regist_port_id(&mut self, port_id: &PortID, xc_id: &XCID) {
        self.portid_to_xcid.insert(*port_id, *xc_id);
    }

    pub fn update_layer_topologies(
        &mut self,
        wxc_route_cands: FxHashMap<SD, Vec<RouteCandidate>>,
        demand_list: &[Demand],
    ) {
        if !UPDATE_LAYER_TOPOLOGIES {
            return;
        }

        self.layer_topologies.insert(
            XCType::Wxc,
            LayerTopology::new_wxc(self, wxc_route_cands, demand_list),
        );

        for xc_type in XCType::iter() {
            if xc_type != XCType::Wxc && self.xcs.values().any(|xc| xc.xc_type == xc_type) {
                self.layer_topologies
                    .insert(xc_type, LayerTopology::new(xc_type, self, demand_list));
            }
        }
    }

    pub fn get_layer_topology(&self, xc_type: &XCType) -> &LayerTopology {
        self.layer_topologies.get(xc_type).unwrap()
    }
}

mod fiber;
pub use fiber::{CoreIndex, Fiber, FiberID, FiberType};

use self::xc::xc_type_to_quality_distance;
pub use self::xc::{XCType, XC};

mod xc;

pub use nw_utils::{network_from_hashmap, wxc_network_from_hashmap};

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum EdgesType {
    Wxc = 0,
    Wbxc = 1,
    Fxc = 2,
    Sxc = 3,
}

use fxhash::{FxHashMap, FxHashSet};
use petgraph::{graph::NodeIndex, Graph};

use crate::{demand::Demand, topology::{self, RouteCandidate}, Edge, Node, SD};

use super::{Network, XCType};

#[derive(Debug, Clone)]
pub struct LayerTopology {
    xc_type: XCType, // Type of Layer
    pub route_cands: FxHashMap<SD, Vec<RouteCandidate>>,
    pub nodes: FxHashSet<Node>,
    pub available_fxc_nodes: FxHashMap<Node, Vec<Node>> 
    // 細粒度レイヤ上でのルーティングによりキーのノードにたどり着けるノードたち
}

impl LayerTopology {
    pub fn new(xc_type: XCType, network: &Network, demand_list: &[Demand]) -> Self {
        let mut layer_top = LayerTopology {
            xc_type,
            route_cands: FxHashMap::default(),
            nodes: FxHashSet::default(),
            available_fxc_nodes: FxHashMap::default(),
        };
        layer_top.update(network, demand_list);
        
        layer_top
    }

    pub fn new_wxc(network: &Network, route_cands: FxHashMap<SD, Vec<RouteCandidate>>, demand_list: &[Demand]) -> Self {
        let nodes = {
            let mut o = FxHashSet::default();
            for sd in route_cands.keys() {
                let (src, dst) = sd.into();
                o.insert(src);
                o.insert(dst);
            }
            o
        };


        let mut available_fxc_nodes = FxHashMap::default();

        for demand in demand_list {
            let mut bypass_entrance_indices = vec![];
            for (idx, fiber_id) in demand.fiber_ids.iter().enumerate() {
                let fiber = network.get_fiber_by_id(fiber_id);
                if network.get_fiber_sd_xc_type(fiber) == [XCType::Wxc, XCType::Fxc] {
                    bypass_entrance_indices.push(idx);
                }
            }

            let mut i = 0;
            for bypass_entrance_idx in bypass_entrance_indices {
                // 直前のバイパスを出てから各バイパスに入るまでに通過したWXCレイヤのノードを列挙する
                // バイパスの入口になるノードはFXCレイヤに存在するノードかつ入口として使用して良いノードになる
                // 列挙されたノードは，WXCレイヤを通過してFXCレイヤ内のノードへ到達しても良いノードになる
                let mut traversed_nodes = vec![];
                let bypass_entrance_node = network.get_fiber_by_id(&demand.fiber_ids[bypass_entrance_idx]).edge.src;
                while i <= bypass_entrance_idx {
                    let fiber = network.get_fiber_by_id(&demand.fiber_ids[i]);
                    if network.get_fiber_sd_xc_type(fiber)[0] == XCType::Wxc {
                        let entry = available_fxc_nodes.entry(fiber.edge.src).or_insert(vec![]);
                        entry.push(bypass_entrance_node);
                        traversed_nodes.push(fiber.edge.src);
                    }
                    i += 1;
                }

            }
        }

        LayerTopology {
            xc_type: XCType::Wxc,
            route_cands,
            nodes,
            available_fxc_nodes,
        }
    }

    fn update(&mut self, network: &Network, demand_list: &[Demand]) {

        let mut g: Graph<usize, usize> = Graph::new();
        for _ in network.get_nodes() {
            g.add_node(1);
        }

        for fiber in network.get_fibers().values() {
            let [src_xc_type, dst_xc_type] = network.get_fiber_sd_xc_type(fiber);
            if (src_xc_type as usize) < (dst_xc_type as usize) && dst_xc_type == self.xc_type {
                let fiber_seq = network.get_fiber_sequence(fiber);
                let edges: Vec<Edge> = fiber_seq.into_iter().map(|fiber_id| network.get_fiber_by_id(&fiber_id).edge).collect();

                g.update_edge(
                    NodeIndex::new(edges.first().unwrap().src.into()), 
                    NodeIndex::new(edges.last().unwrap().dst.into()),
                    1);

            }
        }

        self.update_route_cands(g);
    }

    fn update_route_cands(&mut self, g: Graph<usize, usize>) {
        self.route_cands = topology::get_route_cands_from_graph(g);
    }
}
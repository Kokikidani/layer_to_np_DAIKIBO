use fxhash::FxHashMap;

use crate::{ demand::Demand, network::{Network, XCType}, topology::{RouteCandidate, Topology }, Edge, Node };

use super::{assignemnt_instruction::AssignmentInstruction,get_result_from_route_cand};

const ALPHA: f64 = 1.0;

pub fn search(
    demand: &Demand,
    topology: &Topology,
    network: &mut Network
) -> Option<AssignmentInstruction> {
    let route_cands = topology.route_candidates.get(&demand.sd).unwrap();

    let edge_routes_cost: Vec<f64> = calc_route_cand_costs(network.get_edge_cost(), route_cands,&network);

    // インデックスをソート
    let mut route_cands_index_ordered: Vec<(usize, &f64)> = edge_routes_cost
        .iter()
        .enumerate()
        .collect();
    route_cands_index_ordered.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());

    for (index, _) in route_cands_index_ordered {
        let route_cand: &RouteCandidate = &route_cands[index];

        match get_result_from_route_cand(network, route_cand) {
            Some(result) => return Some(result),
            None => continue,
        }
    }
    None
}

fn calc_route_cand_costs(
    edges_cost: &FxHashMap<Edge, f64>,
    route_cands: &[RouteCandidate],
    network: &Network,
) -> Vec<f64> {
    let mut edge_routes_cost: Vec<f64> = Vec::with_capacity(route_cands.len());

    for route_cand in route_cands {
        let edge_route = &route_cand.edge_route;

        let mut cost: f64 = 0.0;

        for &edge in edge_route {
            
            // ノードID 13の Wxc を通過したかチェック
            let mut alpha = 1.0;
            if edge.src == Node::new(13) || edge.dst == Node::new(13) {
                // Edge から Fiber を取得
                let fiber_ids = network.get_fiber_id_on_edge_partial(&edge);
                for fiber_id in fiber_ids {
                    let fiber = network.get_fiber_by_id(&fiber_id);
                    
                    // Fiber のソースとデスティネーションの XCType を取得
                    let [src_type, dst_type] = fiber.sd_xc_type;
                    
                    // edge.src が Node 13 の場合
                    if edge.src == Node::new(13) && src_type == XCType::Wxc {
                        alpha = ALPHA; // コストを 1.5 倍
                        break; // 該当する場合、早めに終了
                    }
                    
                    // edge.dst が Node 13 の場合
                    if edge.dst == Node::new(13) && dst_type == XCType::Wxc {
                        alpha = ALPHA; // コストを 1.5 倍
                        break; // 該当する場合、早めに終了
                    }
                }
            }

            cost += edges_cost.get(&edge).unwrap_or(&0.0) * alpha;
        }
        
        edge_routes_cost.push(cost);
    }

    edge_routes_cost
}

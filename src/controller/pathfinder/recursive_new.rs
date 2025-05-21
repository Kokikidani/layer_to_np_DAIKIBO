use crate::{demand::Demand, network::{Network, XCType}, topology::{get_shortet_paths, Topology}, SD};

use super::assignemnt_instruction::AssignmentInstruction;

pub fn search(demand: &Demand, topology: &Topology, network: &mut Network) -> Option<AssignmentInstruction> {

    let (src, dst) = &demand.sd.into();

    let wxc_layer_topology = network.get_layer_topology(&XCType::Wxc);
    
    // FXCレイヤで始点ノードを検索
    let mut src_fxc_node_cands = wxc_layer_topology.available_fxc_nodes.get(src).unwrap().to_vec();
    src_fxc_node_cands.sort_by(
        |a,b| 
        get_shortet_paths(topology, &SD::new_from_nodes(*src, *a), None)[0].edge_route.len().cmp(
            &get_shortet_paths(topology, &SD::new_from_nodes(*src, *b), None)[0].edge_route.len()
    ));

    // FXCレイヤで終点ノードを検索
    let mut dst_fxc_node_cands = wxc_layer_topology.available_fxc_nodes.get(dst).unwrap().to_vec();
    dst_fxc_node_cands.sort_by(
        |a,b| 
        get_shortet_paths(topology, &SD::new_from_nodes(*dst, *a), None)[0].edge_route.len().cmp(
            &get_shortet_paths(topology, &SD::new_from_nodes(*dst, *b), None)[0].edge_route.len()
    ));

panic!("例外処理が多すぎるので，保留に");

    let fxc_layer_topology = network.get_layer_topology(&XCType::Fxc);

    if fxc_layer_topology.nodes.contains(&src) {
        // 始点ノードを確定
        todo!()
    } else {
        // srcから最も近いfxc_layerノードを検索



    }



    None
}
use fxhash::FxHashMap;

use crate::{config::Config, debug_println, debugger, network::{FiberID, Network, PortID, XCType}, Edge, WBIndex};

use super::generate_new_fiber;

pub fn expand_wbxc_fibers(config: &Config, network: &mut Network, target_edges: &[Edge]) {
    
    if target_edges.len() < 2 {
        panic!("`target_edges` should be longer than two");
    }

    let (wb_index, fiber_seq) = get_min_expand_wb_fiber_sequences(network, target_edges);
    let mut prev_dst_port_id = PortID::nil();

    for (idx, (fiber_id, target_edge)) in fiber_seq.iter().zip(target_edges.iter()).enumerate() {
        let target_fiber = if let Some(target_fiber_id) = fiber_id {
            network.get_fiber_by_id(target_fiber_id)
        } else {
            let fiber = if idx == 0 {
                generate_new_fiber(network, target_edge, XCType::Wxc, XCType::Wbxc)
            } else if idx == fiber_seq.len() - 1 {
                generate_new_fiber(network, target_edge, XCType::Wbxc, XCType::Wxc)
            } else {
                generate_new_fiber(network, target_edge, XCType::Wbxc, XCType::Wbxc)
            };
            debugger::log_fiber_expand(config, network, &fiber);
            network.regist_fiber(fiber)
        };

        let target_fiber_src_port_id = target_fiber.src_port_ids[0];
        let target_fiber_dst_port_id = target_fiber.dst_port_ids[0];

        if idx != 0 {
            let xc = network.get_xc_mut_on_node(target_edge.src.into(), &XCType::Wbxc);
            xc.connect_io_wb(&prev_dst_port_id, &target_fiber_src_port_id, &wb_index).unwrap_or_else(|_err| {
                debug_println!(prev_dst_port_id, target_fiber_src_port_id);
                debug_println!(target_edge, target_edges);
                debug_println!(network.get_fiber_by_src_device(&target_fiber_src_port_id));
                debug_println!(network.get_xc_by_input_port_id(&prev_dst_port_id));
                debug_println!(network.get_xc_by_output_port_id(&target_fiber_src_port_id));
                panic!();
            });
        }
        prev_dst_port_id = target_fiber_dst_port_id;
    }

    debugger::log_wb_bypass(config, target_edges, &wb_index);
}

fn get_fiber_contains_unused_wb_specified(network: &Network, fiber_ids: &[FiberID], wb_index: &WBIndex, sd_xc_type: &[XCType; 2]) -> Option<FiberID> {

    for fiber_id in fiber_ids {
        let fiber = network.get_fiber_by_id(fiber_id);
        if network.get_fiber_sd_xc_type(fiber) == *sd_xc_type {
            let src_xc = network.get_xc_by_output_port_id(&fiber.src_port_ids[0]);
            match src_xc.xc_type {
                XCType::Wxc | XCType::Fxc | XCType::Sxc| XCType::Added_Wxc => (),
                XCType::Wbxc => {
                    if src_xc.is_output_device_wb_occupied(&fiber.src_port_ids[0], wb_index) {
                        continue;
                    }
                },
            }
            
            let dst_xc = network.get_xc_by_input_port_id(&fiber.dst_port_ids[0]);
            match dst_xc.xc_type {
                XCType::Wxc | XCType::Fxc | XCType::Sxc| XCType::Added_Wxc => (),
                XCType::Wbxc => {
                    if dst_xc.is_input_device_wb_occupied(&fiber.dst_port_ids[0], wb_index) {
                        continue;
                    }
                },
            }

            return Some(*fiber_id)
        }
    }

    None
}

fn get_min_expand_wb_fiber_sequences(network: &Network, target_edges: &[Edge]) -> (WBIndex, Vec<Option<FiberID>>) {
    let mut fiber_sequences = FxHashMap::default();

    for wb_index in WBIndex::iter() {
        let entry = fiber_sequences.entry(wb_index).or_insert(vec![]);

        for (idx, edge) in target_edges.iter().enumerate() {
            let fiber_ids_on_edge = network.get_fiber_id_on_edge(edge);
            if idx == 0{
                entry.push(get_fiber_contains_unused_wb_specified(network, &fiber_ids_on_edge, &wb_index, &[XCType::Wxc, XCType::Wbxc]));
            } else if idx == target_edges.len() - 1 {
                entry.push(get_fiber_contains_unused_wb_specified(network, &fiber_ids_on_edge, &wb_index, &[XCType::Wbxc, XCType::Wxc]));
            } else {
                entry.push(get_fiber_contains_unused_wb_specified(network, &fiber_ids_on_edge, &wb_index, &[XCType::Wbxc, XCType::Wbxc]));
            }
        }

        if !entry.contains(&None) {
            break;
        }
    }

    let mut wb_fiber_seq: Vec<(WBIndex, Vec<Option<FiberID>>)> = fiber_sequences.into_iter().collect();
    wb_fiber_seq.sort_by_key(|(a_wb, _)| a_wb.index());
    wb_fiber_seq.sort_by_key(|(_a_wb, a_fiber_seq)| a_fiber_seq.iter().filter(|x| x.is_none()).count());

    let (wb_index, fiber_seq) = wb_fiber_seq.first().unwrap();
    (*wb_index, fiber_seq.clone())
}

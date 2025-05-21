use assignemnt_instruction::AssignmentInstruction;

use crate::{
    config::Config, demand::Demand, network::{ CoreIndex, FiberID, Network, XCType }, np_core::StateMatrix, topology::{ RouteCandidate, Topology }, utils::contains_subslice, WBIndex
};

mod assignemnt_instruction;
mod ff;
mod ff_randomized;
mod rd;
mod rd_da;

mod recursive_new;

const SHORTCUT: bool = true;

pub fn search(
    config: &Config,
    demand: &Demand,
    topology: &Topology,
    network: &mut Network
) -> Option<AssignmentInstruction> {
    match config.policy.routing_policy.as_str() {
        "FF" | "ff"       =>            ff::search(demand, topology, network),
        "ff_randomized"   => ff_randomized::search(demand, topology, network),
        "RD" | "rd"       =>            rd::search(demand, topology, network),
        "RD_DA" | "rd_da" =>         rd_da::search(demand, topology, network),
        "layer_search"    => recursive_new::search(demand, topology, network),
        _ => panic!("Invalid Routing Policy: {}.", config.policy.routing_policy),
    }
}

pub fn calc_fiber_route_score(network: &Network, fiber_route: &[FiberID]) -> usize {
    let mut score = 0;
    for fiber_id in fiber_route {
        let [src_type, dst_type] = network.get_fiber_sd_xc_type_by_id(fiber_id);

        match src_type {
            XCType::Wxc  => (),
            XCType::Added_Wxc  => (),
            XCType::Wbxc => score +=  10,
            XCType::Fxc  => score +=  10,
            XCType::Sxc  => score +=  10,
        }

        match dst_type {
            XCType::Wxc  => (),
            XCType::Added_Wxc  => (),
            XCType::Wbxc => score +=  10,
            XCType::Fxc  => score +=  10,
            XCType::Sxc  => score +=  10,
        }
    }

    score
}

fn get_result_from_route_cand(network: &Network, route_cand: &RouteCandidate) -> Option<AssignmentInstruction> {
    let fiber_core_route_cands: Vec<(Vec<FiberID>, Vec<CoreIndex>)> = get_empty_fiber_core_routes(network, route_cand, 1);

    #[allow(clippy::never_loop)]
    for (fiber_route, core_indices) in &fiber_core_route_cands {
        let mut target_state_matrix = StateMatrix::new();
        let mut flag = true;

        for (fiber_id, core_index) in fiber_route.iter().zip(core_indices.iter()) {
            let fiber = network.get_fiber_by_id(fiber_id);
            let state_matrix_of_fiber_core = fiber.state_matrixes[core_index.index()];
            // println!("STATEMAT| {target_state_matrix}");

            target_state_matrix |= state_matrix_of_fiber_core;

            if !target_state_matrix.has_empty_contiguous_slots(1) {
                flag = false;
                break;
            }
        }

        if flag {
            
            'slot_loop: for (slot, s) in target_state_matrix.iter().enumerate() {
                if !*s {
                    for target_fiber_id in fiber_route.iter() {
                        let target_fiber = network.get_fiber_by_id(target_fiber_id);
                        if target_fiber.sd_xc_type == [XCType::Wxc, XCType::Wbxc] {
                            if let Some(fiber_seq) = network.get_fiber_sequence_wb(target_fiber, &WBIndex::from_wavelength(slot)) {
                                if !contains_subslice(fiber_route, &fiber_seq) {
                                    continue 'slot_loop;
                                }
                            } else {
                                continue 'slot_loop;
                            }
                        }
                    }

                    return Some(
                        AssignmentInstruction {
                            fiber_ids: fiber_route.clone(),
                            slot_head: vec![slot; fiber_route.len()],
                            slot_width: 1,
                            core_indices: core_indices.clone(),
                        }
                    );
                }

                }
            }
            panic!();
        }

    None
}

fn get_empty_fiber_core_routes(
    network: &Network,
    route_cand: &RouteCandidate,
    width: usize
) -> Vec<(Vec<FiberID>, Vec<CoreIndex>)> {
    // Final result of this function
    let mut result_fiber_core_routes: Vec<(Vec<FiberID>, Vec<CoreIndex>)> = vec![];

    // Variables used in recursive function
    let mut target_fiber_route: Vec<FiberID> = vec![];
    let mut target_core_indices: Vec<CoreIndex> = vec![];
    let mut target_state_matrix: StateMatrix = StateMatrix::new();

    // Get fiber_ids on edges on the route
    let mut fiber_ids_on_edges = vec![];
    for edge in &route_cand.edge_route {
        fiber_ids_on_edges.push(network.get_fiber_ids_on_edge_empty(edge));
        // fiber_ids_on_edges.push(network.get_fiber_id_on_edge_partial(edge));
    }

    let _ = get_empty_fiber_core_routes_recursive(
        network,
        &fiber_ids_on_edges,
        width,
        &mut result_fiber_core_routes,
        &mut target_fiber_route,
        &mut target_state_matrix,
        &mut target_core_indices
    );

    if !SHORTCUT {
        let fiber_route_scores: Vec<usize> = result_fiber_core_routes
            .iter()
            .map(|(x_fiber_route, _x_core_indices)| calc_fiber_route_score(network, x_fiber_route))
            .collect();

        #[allow(clippy::type_complexity)]
        let mut combined: Vec<(&(Vec<FiberID>, Vec<CoreIndex>), &usize)> = result_fiber_core_routes
            .iter()
            .zip(fiber_route_scores.iter())
            .collect();
        combined.sort_by(|a: &(&(Vec<FiberID>, Vec<CoreIndex>), &usize), b| b.1.cmp(a.1));
        result_fiber_core_routes = combined
            .iter()
            .map(|x: &(&(Vec<FiberID>, Vec<CoreIndex>), &usize)| (x.0.0.to_vec(), x.0.1.to_vec()))
            .collect();
    }

    result_fiber_core_routes
}

fn get_empty_fiber_core_routes_recursive(
    network: &Network,
    fiber_ids_on_edges: &[Vec<FiberID>],
    width: usize,
    result_fiber_core_routes: &mut Vec<(Vec<FiberID>, Vec<CoreIndex>)>,
    target_fiber_route: &mut Vec<FiberID>,
    target_state_matrix: &mut StateMatrix,
    target_core_indices: &mut Vec<CoreIndex>,
) -> Result<(), ()> {
    let target_level = target_fiber_route.len();
    let final_level = fiber_ids_on_edges.len() - 1;

    let fiber_ids_on_target_edge = &fiber_ids_on_edges[target_level];
    
    let mut fiber_ids_on_target_edge_sorted = fiber_ids_on_target_edge.clone();
    fiber_ids_on_target_edge_sorted.sort_by(|a, b| (network.get_fiber_sd_xc_type_by_id(b)[1] as usize).cmp(&(network.get_fiber_sd_xc_type_by_id(a)[1] as usize)));


    for target_fiber_id in &fiber_ids_on_target_edge_sorted {
        if target_level == 0 {
            // Check first XC type
            let first_fiber = network.get_fiber_by_id(target_fiber_id);
            let first_xc_type = network.get_fiber_sd_xc_type(first_fiber)[0];
            if first_xc_type != XCType::Wxc {
                continue; // To next fiber
            }
        }

        if target_level == final_level {
            // Check last XC type
            let last_fiber = network.get_fiber_by_id(target_fiber_id);
            let last_xc_type = network.get_fiber_sd_xc_type(last_fiber)[1];
            if last_xc_type != XCType::Wxc {
                continue; // To next fiber
            }
        }

        let [src_xc_type, dst_xc_type] = network.get_fiber_sd_xc_type_by_id(target_fiber_id);
        match [src_xc_type, dst_xc_type] {            
            [XCType::Wxc, XCType::Fxc] | [XCType::Wxc, XCType::Sxc]| [XCType::Added_Wxc, XCType::Fxc] => {
                
                let target_core_index = CoreIndex::new(0);
                if !check_continuity(network, target_fiber_route, target_fiber_id, &target_core_index) {
                    continue;
                }
                
                let core_factor = network.get_fiber_core_factor_by_id(target_fiber_id);
                for target_core_index_as_usize in 0..core_factor {
                    let target_core_index: CoreIndex = target_core_index_as_usize.into();

                    // Mask Slots
                    let state_matrix_of_target_fiber_core = network.get_fiber_by_id(target_fiber_id).state_matrixes[target_core_index_as_usize];
                    let new_target_state_matrix = *target_state_matrix | state_matrix_of_target_fiber_core;
                    let tmp_target_state_matrix = *target_state_matrix;

                    // Check Slots
                    match new_target_state_matrix.get_empty_contiguous_slots(width) {
                        None => continue,
                        Some(_) => {
                            target_fiber_route.push(*target_fiber_id);
                            target_core_indices.push(target_core_index);
                            target_state_matrix.clone_from(&new_target_state_matrix);

                            if target_level == final_level {
                                result_fiber_core_routes.push((
                                    target_fiber_route.clone(),
                                    target_core_indices.clone()
                                ));

                                if SHORTCUT { return Ok(()) }

                                target_fiber_route.pop();
                                target_core_indices.pop();
                                target_state_matrix.clone_from(&tmp_target_state_matrix);

                                continue;
                            }

                            match get_empty_fiber_core_routes_recursive(network, fiber_ids_on_edges, width, result_fiber_core_routes, target_fiber_route, target_state_matrix, target_core_indices) {
                                Ok(_) => {
    
                                    if SHORTCUT { return Ok(()) }

                                    target_fiber_route.pop();
                                    target_core_indices.pop();
                                    target_state_matrix.clone_from(&tmp_target_state_matrix);
                                    continue;
                                },
                                Err(_) => {
                                    target_fiber_route.pop();
                                    target_core_indices.pop();
                                    target_state_matrix.clone_from(&tmp_target_state_matrix);
                                    continue;
                                },
                            }
                        },
                    }
                }                        
            },
            [XCType::Wxc, XCType::Wxc] | [XCType::Sxc, XCType::Wxc] | [XCType::Sxc, XCType::Sxc] | [XCType::Fxc, XCType::Wxc] | [XCType::Fxc, XCType::Fxc]=> {

                let target_core_index = if src_xc_type == XCType::Wxc {
                    CoreIndex::new(0)
                } else {
                    *target_core_indices.last().unwrap_or(&CoreIndex::new(0))
                };
                
                if !check_continuity(network, target_fiber_route, target_fiber_id, &target_core_index) {
                    continue;
                }

                {
                    // Mask Slots
                    let state_matrix_of_target_fiber_core = network.get_fiber_by_id(target_fiber_id).state_matrixes[target_core_index.index()];
                    let new_target_state_matrix = *target_state_matrix | state_matrix_of_target_fiber_core;
                    let tmp_target_state_matrix = *target_state_matrix;

                    // Check Slots
                    match new_target_state_matrix.get_empty_contiguous_slots(width) {
                        None => continue,
                        Some(_) => {
                            target_fiber_route.push(*target_fiber_id);
                            target_core_indices.push(target_core_index);
                            target_state_matrix.clone_from(&new_target_state_matrix);

                            if target_level == final_level {
                                result_fiber_core_routes.push((
                                    target_fiber_route.clone(),
                                    target_core_indices.clone()
                                ));
                                
                                if SHORTCUT { return Ok(()) }

                                target_fiber_route.pop();
                                target_core_indices.pop();
                                target_state_matrix.clone_from(&tmp_target_state_matrix);

                                continue;
                            }

                            match get_empty_fiber_core_routes_recursive(network, fiber_ids_on_edges, width, result_fiber_core_routes, target_fiber_route, target_state_matrix, target_core_indices) {
                                Ok(_) => {

                                    if SHORTCUT { return Ok(()) }

                                    target_fiber_route.pop();
                                    target_core_indices.pop();
                                    target_state_matrix.clone_from(&tmp_target_state_matrix);
                                    continue;
                                },
                                Err(_) => {
                                    target_fiber_route.pop();
                                    target_core_indices.pop();
                                    target_state_matrix.clone_from(&tmp_target_state_matrix);
                                    continue;
                                },
                            }
                        },
                    }
                }
            },
            [XCType::Wbxc, XCType::Wxc] | [XCType::Wbxc, XCType::Wbxc] => {
                
                let target_core_index = if src_xc_type == XCType::Wxc {
                    CoreIndex::new(0)
                } else {
                    *target_core_indices.last().unwrap()
                };
                
                for wb_index in WBIndex::iter() {
                    if !check_continuity_wb(network, target_fiber_route, target_fiber_id, &wb_index) {
                        continue;
                    }

                    // Mask Slots
                    let state_matrix_of_target_fiber_core = network.get_fiber_by_id(target_fiber_id).state_matrixes[0];
                    let mut new_target_state_matrix = *target_state_matrix | state_matrix_of_target_fiber_core;
                    new_target_state_matrix.apply_witout_wb_filter(&wb_index);
                    let tmp_target_state_matrix = *target_state_matrix;

                    // Check Slots (WaveBand Check)
                    if width != 1 { unimplemented!(); }

                    match new_target_state_matrix.get_empty_contiguous_slots(1) {
                        None => continue,
                        Some(_) => {
                            target_fiber_route.push(*target_fiber_id);
                            target_core_indices.push(target_core_index);
                            target_state_matrix.clone_from(&new_target_state_matrix);

                            if target_level == final_level {
                                result_fiber_core_routes.push((
                                    target_fiber_route.clone(),
                                    target_core_indices.clone()
                                ));

                                if SHORTCUT { return Ok(()) }

                                target_fiber_route.pop();
                                target_core_indices.pop();
                                target_state_matrix.clone_from(&tmp_target_state_matrix);
                                continue;
                            }

                            match get_empty_fiber_core_routes_recursive(network, fiber_ids_on_edges, width, result_fiber_core_routes, target_fiber_route, target_state_matrix, target_core_indices) {
                                Ok(_) => {
                                    if SHORTCUT { return Ok(()) }

                                    target_fiber_route.pop();
                                    target_core_indices.pop();
                                    target_state_matrix.clone_from(&tmp_target_state_matrix);
                                    continue;
                                },
                                Err(_) => {
                                    target_fiber_route.pop();
                                    target_core_indices.pop();
                                    target_state_matrix.clone_from(&tmp_target_state_matrix);
                                    continue;
                                },
                            }
                        },
                    }
                }
            }
            [XCType::Wxc, XCType::Wbxc] => {

                let target_core_index = CoreIndex::new(0);

                for wb_index in WBIndex::iter() {
                    if !check_continuity(network, target_fiber_route, target_fiber_id, &target_core_index) {
                        continue;
                    }

                    // Mask Slots
                    let state_matrix_of_target_fiber_core = network.get_fiber_by_id(target_fiber_id).state_matrixes[0];
                    let mut new_target_state_matrix = *target_state_matrix | state_matrix_of_target_fiber_core;
                    new_target_state_matrix.apply_witout_wb_filter(&wb_index);
                    let tmp_target_state_matrix = *target_state_matrix;

                    // Check Slots (WaveBand Check)
                    if width != 1 { unimplemented!(); }

                    match new_target_state_matrix.get_empty_contiguous_slots(1) {
                        None => continue,
                        Some(_) => {
                            target_fiber_route.push(*target_fiber_id);
                            target_core_indices.push(target_core_index);
                            target_state_matrix.clone_from(&new_target_state_matrix);

                            if target_level == final_level {
                                result_fiber_core_routes.push((
                                    target_fiber_route.clone(),
                                    target_core_indices.clone()
                                ));

                                if SHORTCUT { return Ok(()) }

                                target_fiber_route.pop();
                                target_core_indices.pop();
                                target_state_matrix.clone_from(&tmp_target_state_matrix);
                                continue;
                            }

                            match get_empty_fiber_core_routes_recursive(network, fiber_ids_on_edges, width, result_fiber_core_routes, target_fiber_route, target_state_matrix, target_core_indices) {
                                Ok(_) => {
                                    if SHORTCUT { return Ok(()) }

                                    target_fiber_route.pop();
                                    target_core_indices.pop();
                                    target_state_matrix.clone_from(&tmp_target_state_matrix);
                                    continue;
                                },
                                Err(_) => {
                                    target_fiber_route.pop();
                                    target_core_indices.pop();
                                    target_state_matrix.clone_from(&tmp_target_state_matrix);
                                    continue;
                                },
                            }
                        },
                    }
                }
            }
            _ => unimplemented!("{}, {}", src_xc_type, dst_xc_type)
        }
    }

    Err(())
}

fn check_continuity_wb(network: &Network, target_fiber_route: &[FiberID], target_fiber_id: &FiberID, target_wb_index: &WBIndex) -> bool {

    if let Some(prev_fiber_id) = target_fiber_route.last() {
        let prev_fiber = network.get_fiber_by_id(prev_fiber_id);
        let target_fiber = network.get_fiber_by_id(target_fiber_id);

        let input_id = &prev_fiber.dst_port_ids[0];
        let output_id = &target_fiber.src_port_ids[0];

        // Check matching of xc id
        let input_parent_xc = network.get_xc_by_input_port_id(input_id);
        let output_parent_xc = network.get_xc_by_output_port_id(output_id);
        if input_parent_xc.id != output_parent_xc.id {
            return false;
        }

        let xc = network.get_xc_by_io_device(input_id, output_id);
        if !xc.can_route_wb(input_id, output_id, target_wb_index) {
            return false;
        }
    }

    true
}

fn check_continuity(network: &Network, target_fiber_route: &[FiberID], target_fiber_id: &FiberID, target_core_index: &CoreIndex) -> bool {
    // Check continuity
    if let Some(prev_fiber_id) = target_fiber_route.last() {
        let prev_fiber   = network.get_fiber_by_id(prev_fiber_id);
        let target_fiber = network.get_fiber_by_id(target_fiber_id);

        if prev_fiber.get_core_num() <= target_core_index.index() || target_fiber.get_core_num() <= target_core_index.index() {
            panic!();
        }

        let input_ids  = &prev_fiber.dst_port_ids;
        let output_ids = &target_fiber.src_port_ids;

        // Check matching of xc id
        let input_parent_xc  = network.get_xc_by_input_port_id(&input_ids[target_core_index.index()]);
        let output_parent_xc = network.get_xc_by_output_port_id(&output_ids[target_core_index.index()]);
        if input_parent_xc.id != output_parent_xc.id {
            return false;
        }

        // Check status of XC (can route?)
        // let xc = network.get_xc_by_io_device(&input_ids[target_core_index.index()], &output_ids[target_core_index.index()]);
        let xc = input_parent_xc;
        if !xc.can_route(&input_ids[target_core_index.index()], &output_ids[target_core_index.index()]) {
            return false;
        }
    } // END Check continuity

    true
}
use crate::SLOT;

use super::{ Network, XCType };

/// WXC2WXCをのぞいたstate_matrixを返す
pub fn state_matrix_wo_w2w_from_network(network: &Network) -> Vec<[bool; SLOT]> {
    let mut state_matrix: Vec<[bool; SLOT]> = vec![];
    for fiber in network.fibers.values() {
        if network.get_fiber_sd_xc_type(fiber) != [XCType::Wxc, XCType::Wxc] {
            state_matrix.push(fiber.state_matrixes[0].get_raw());
        }
    }

    state_matrix
}

/// state_matrix_labelを返す
/// log_state_matrix向け
pub fn state_matrix_label_wo_w2w_from_network(network: &Network) -> Vec<(usize, usize)> {
    let mut state_matrix_label: Vec<(usize, usize)> = vec![];

    for fiber in network.fibers.values() {
        if network.get_fiber_sd_xc_type(fiber) != [XCType::Wxc, XCType::Wxc] {
            state_matrix_label.push((fiber.edge.src.into(), fiber.edge.dst.into()));
        }
    }

    state_matrix_label
}

use crate::network::{Network, XCType };

pub fn calc_fiber_count_ratio(network: &Network, conv_nw_w2w_fiber_count: usize) -> f64 {

    let mut sum = 0;

    for fiber in network.get_fibers().values() {
        // Something to SXC については，使用中コアのみカウント (Bundled SCFとして扱う)

        let [src_xc_type, dst_xc_type] = network.get_fiber_sd_xc_type(fiber);
        if (src_xc_type == XCType::Sxc && dst_xc_type != XCType::Sxc) || (src_xc_type != XCType::Sxc && dst_xc_type == XCType::Sxc) {
            let unused_cores = network.get_unused_core(fiber);
            sum += fiber.get_core_num() - unused_cores.len();
        } else {
            sum += fiber.get_core_num();
        }
    }

    (sum as f64) / (conv_nw_w2w_fiber_count as f64)
}

pub fn calc_max_wxc_size(network: &Network) -> usize {
    
    let mut wxc_scale = 0;

    for node in network.get_nodes() {
        if let Some(wxc) = network.get_xc_on_node(node.into(), &XCType::Wxc) {
            if wxc_scale < wxc.get_size() {
                wxc_scale = wxc.get_size();
            }
        }
    }

    wxc_scale
}


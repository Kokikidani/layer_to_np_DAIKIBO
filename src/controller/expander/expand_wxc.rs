use crate::{config::Config, debugger, network::{Network, XCType}, Edge};

use super::generate_new_fiber;

pub fn expand_wxc_fibers(config: &Config, network: &mut Network, target_edges: &[Edge]) {
    let mut fibers = vec![];

    for edge in target_edges {
        let fiber = generate_new_fiber(network, edge, XCType::Wxc, XCType::Wxc);
        fibers.push(fiber);
    }

    debugger::log_fibers_expand(config, network, &fibers);

    network.regist_fibers(fibers);
}

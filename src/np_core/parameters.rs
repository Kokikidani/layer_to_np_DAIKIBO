pub const SLOT: usize = 96;
pub const MIN_BYPASS_LEN: usize = 2;
pub const MAX_BYPASS_LEN: usize = 4;
pub const THREADS: usize = 16;

pub const CURVE_RANGE_BOTTOM: f64 = 1.0;
pub const CURVE_RANGE_UP: f64 = 0.5;

// For
pub const PB_TEMPLATES: &str =
    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta_precise}) \t{msg}";
pub const PB_CHARS: &str = "#9876543210>-";
pub const MEAN_N: usize = 96;

pub const SHORTEST_K: usize = 100;
pub const HOP_SLUG: usize = 2;

pub const WXC_PORT_Q_DISTANCE: usize = 25;
pub const FXC_PORT_Q_DISTANCE: usize = 50;

pub const CURVE_GRAPH_SCRIPT: &str = "./scripts/blocking_curve.py";
pub const TRAVERSE_GRAPH_SCRIPT: &str = "./scripts/wxc_port_traverse_count.py";

pub const WAVEBAND_COUNT: usize = 4;

pub const CORE_FACTOR: usize = 4;
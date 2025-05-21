use indicatif::{ProgressBar, ProgressStyle};
use petgraph::{ algo::all_simple_paths, graph::{ Graph, NodeIndex } };
use rayon::{iter::{IntoParallelIterator, ParallelIterator}, ThreadPoolBuilder};
use std::{ cmp::max, fs::File, io::Read };
use itertools::iproduct;

use crate::{ config::Config, np_core::parameters::{ HOP_SLUG, PB_CHARS, PB_TEMPLATES, SHORTEST_K, THREADS }, utils::{self, shuffle_array}, Edge, SD };

use fxhash::FxHashMap;

#[derive(Debug, Clone)]
pub struct RouteCandidate {
    pub node_route: Vec<usize>,
    pub edge_route: Vec<Edge>,
}
impl RouteCandidate {
    pub fn new(node_route: Vec<usize>, edge_route: Vec<Edge>) -> RouteCandidate {
        RouteCandidate {
            node_route,
            edge_route,
        }
    }
}

#[derive(Debug)]
pub struct Topology {
    /// トポロジの名前
    pub name: String,
    /// 隣接行列 (多分、必要ない)
    pub link_matrix: Vec<Vec<bool>>,
    /// エッジ
    pub edges: Vec<Edge>,
    /// ルート情報
    pub route_candidates: FxHashMap<SD, Vec<RouteCandidate>>,
}

impl Topology {
    pub fn new(config: &Config) -> Topology {
        let name = config.network.topology.clone();
        let link_matrix = get_link_matrix(&name);
        let edges = link_matrix_to_edges(&link_matrix);

        let route_candidates = get_route_candidates_from_matrix(&link_matrix);

        Topology {
            name,
            link_matrix,
            edges,
            route_candidates,
        }
    }
}



fn get_route_candidates_from_matrix(link_matrix: &[Vec<bool>]) -> FxHashMap<SD, Vec<RouteCandidate>> {

    // グラフの作成
    let mut g = Graph::<usize, usize>::new();

    for _ in 0..link_matrix.len() {
        g.add_node(1);
    }

    let edges = link_matrix_to_edges(link_matrix);

    for edge in edges {
        g.add_edge(NodeIndex::new(edge.src.into()), NodeIndex::new(edge.dst.into()), 1);
    }
    
    get_route_cands_from_graph(g)
}

pub fn get_route_cands_from_graph(g: Graph<usize, usize>) -> FxHashMap<SD, Vec<RouteCandidate>> {
    let sd_pairs: Vec<(NodeIndex, NodeIndex)> = iproduct!(g.node_indices(), g.node_indices()).filter(|(s,d)| s.index() != d.index()).collect();
    
    let pb = ProgressBar::new(sd_pairs.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar().template(PB_TEMPLATES).unwrap().progress_chars(PB_CHARS)
    );

    let pool = ThreadPoolBuilder::new()
        .num_threads(THREADS)
        .build()
        .expect("Failed to create thread pool");
    let route_candidates_vec: Vec<(SD, Vec<RouteCandidate>)> = pool.install(|| {
        sd_pairs.into_par_iter().map(|(src, dst)| {
            
            let mut route_all: Vec<Vec<NodeIndex>> = vec![];
            let mut route_length = 0;

            let mut shortest_route_len = None;

            while shortest_route_len.is_none() || route_length <= shortest_route_len.unwrap() + HOP_SLUG {
                if g.node_count() < route_length {
                    break;
                }

                let routes = all_simple_paths::<Vec<_>, _>(
                    &g,
                    src,
                    dst,
                    route_length,
                    Some(route_length)
                ).collect::<Vec<_>>();
                
                if shortest_route_len.is_none() && !routes.is_empty(){
                    shortest_route_len = Some(route_length);
                }

                route_all.extend(routes);
                route_length += 1;
            }

            if route_all.is_empty() {
                pb.inc(1);
                (SD::new(src.index(), dst.index()), vec![])
            } else {
                // shortest_k 打ち切り
                route_all.truncate(SHORTEST_K);

                // debug::alert_route_cands_parameter

                // hop_slug 打ち切り
                let shortest_route_length = route_all[0].len();
                let mut truncate_index = SHORTEST_K;
                for (route_all_index, route) in route_all.iter().enumerate() {
                    if route.len() > shortest_route_length + HOP_SLUG {
                        truncate_index = route_all_index;
                        break;
                    }
                }
                route_all.truncate(truncate_index);

                // NodeIndex > usize
                let mut tmp_all: Vec<Vec<usize>> = vec![];
                for route in route_all {
                    let mut tmp: Vec<usize> = vec![];
                    for node in route {
                        tmp.push(node.index());
                    }
                    tmp_all.push(tmp);
                }

                // 候補追加
                let mut o2 = vec![];
                for tmp in tmp_all {
                    let edge_route = {
                        let node_route: &[usize] = &tmp;
                        let mut out = vec![];

                        for edge in node_route.windows(2).collect::<Vec<_>>() {
                            out.push(Edge::new(edge[0], edge[1]));
                        }

                        out
                    };
                    let route_candidate = RouteCandidate::new(tmp, edge_route);
                    o2.push(route_candidate);
                }

                pb.inc(1);

                (SD::new(src.index(), dst.index()), o2)
            }
        })
    }).collect();

    let mut route_candidates = FxHashMap::default();
    for (sd, route_candidate) in route_candidates_vec {
        route_candidates.insert(sd, route_candidate);
    }
    route_candidates
}

fn link_matrix_to_edges(link_matrix: &[Vec<bool>]) -> Vec<Edge> {
    let mut o = vec![];
    for (r, l) in link_matrix.iter().enumerate() {
        for (c, v) in l.iter().enumerate() {
            if *v {
                o.push(Edge::new(r, c));
            }
        }
    }

    o
}

fn get_link_matrix(name: &str) -> Vec<Vec<bool>> {
    let file_name = format!("./files/topology/{}.txt", name).to_lowercase();
    match File::open(file_name) {
        Ok(mut file) => {
            let mut content = String::new();
            match file.read_to_string(&mut content) {
                Ok(_) => utils::string_to_vec2_bool(&content),
                Err(_) => panic!("ファイルを読み込めませんでした"),
            }
        }
        Err(_) => panic!("ファイルを開けませんでした"),
    }
}

pub fn get_ave_shortest_hops(topology: &Topology) -> f64 {
    let mut sum_hops = 0;
    for route_cands in topology.route_candidates.values() {
        let shortest_hops = route_cands[0].edge_route.len();
        sum_hops += shortest_hops;
    }

    (sum_hops as f64) / (topology.route_candidates.len() as f64)
}

pub fn get_fixed_shortest_path(topology: &Topology, sd: &SD, min_len: Option<usize>) -> RouteCandidate {
    let shortest_paths = get_shortet_paths(topology, sd, min_len);
    shortest_paths[0].clone()
}

pub fn get_random_shortest_path(topology: &Topology, sd: &SD, rand_seed: u64, min_len: Option<usize>) -> RouteCandidate {
    let mut shortest_paths = get_shortet_paths(topology, sd, min_len);
    shuffle_array(&mut shortest_paths, rand_seed);
    shortest_paths[0].clone()
}

pub fn get_shortet_paths<'a>(topology: &'a Topology, sd: &'a SD, min_len: Option<usize>) -> Vec<&'a RouteCandidate> {
    let route_cands = topology.route_candidates.get(sd).unwrap();

    let shortest_path_len = max(route_cands[0].edge_route.len(), min_len.unwrap_or(0));

    let route_cands_slices: Vec<&RouteCandidate> = route_cands
            .iter()
            .filter(|p| p.edge_route.len() == shortest_path_len)
            .collect();

    if route_cands_slices.is_empty() {
        panic!("");
    }

    route_cands_slices
}
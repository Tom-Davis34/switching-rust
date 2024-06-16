use std::cmp;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::iter::Filter;
use std::iter::zip;
use std::rc::Rc;
use std::slice::Iter;
use std::str::FromStr;

use crate::graph::AdjacentInfo;
use crate::graph::Edge;
use crate::graph::EdgeIndex;
use crate::graph::Graph;
use crate::graph::NodeIndex;
use crate::graph::plague_algo::SigAlg;
use crate::graph::plague_algo::plague_algo_pure;
use crate::power_system::EdgeData::Cir;
use crate::power_system::EdgeData::Sw;
use crate::traits::C32;

use crate::graph::plague_algo::generate_sigma_alg;

use self::file_parsing::FileEdge;

mod file_parsing;
pub mod outage;
pub mod power_flow_model;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum U {
    Open,
    Closed,
    DontCare,
}

impl fmt::Display for U {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl U {
    pub fn hamming_dist(target_u: &Vec<U>, actual_u: &Vec<U>) -> f32 {
        zip(target_u.iter(), actual_u.iter())
            .map(|(t_u, a_u)| match t_u {
                U::Open => {
                    if a_u == &U::Closed {
                        1.0
                    } else {
                        0.0
                    }
                }
                U::Closed => {
                    if a_u == &U::Open {
                        1.0
                    } else {
                        0.0
                    }
                }
                U::DontCare => 0.0,
            })
            .sum()
    }

    pub fn not(&self) -> U {
        match self {
            U::Open => U::Closed,
            U::Closed => U::Open,
            U::DontCare => panic!(),
        }
    }
}

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum NodeType {
    PQ,
    PV,
    Sk,
}

impl Ord for NodeType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (&self, other) {
            (NodeType::PQ, NodeType::PQ) => cmp::Ordering::Equal,
            (NodeType::PQ, NodeType::PV) => cmp::Ordering::Less,
            (NodeType::PQ, NodeType::Sk) => cmp::Ordering::Less,
            (NodeType::PV, NodeType::PQ) => cmp::Ordering::Greater,
            (NodeType::PV, NodeType::PV) => cmp::Ordering::Equal,
            (NodeType::PV, NodeType::Sk) => cmp::Ordering::Less,
            (NodeType::Sk, NodeType::PQ) => cmp::Ordering::Greater,
            (NodeType::Sk, NodeType::PV) => cmp::Ordering::Greater,
            (NodeType::Sk, NodeType::Sk) => cmp::Ordering::Equal,
        }
    }

    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        cmp::max_by(self, other, Ord::cmp)
    }

    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        cmp::min_by(self, other, Ord::cmp)
    }

    fn clamp(self, _min: Self, _max: Self) -> Self
    where
        Self: Sized,
        Self: PartialOrd,
    {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DeltaU {
    pub index: EdgeIndex,
    pub new_u: U,
}

impl Display for DeltaU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.new_u)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError;

#[derive(PartialEq, Clone)]
pub struct PsNode {
    pub num: usize,
    pub index: NodeIndex,
    pub load: C32,
    pub gen: C32,
    pub system_v: f32,
    pub n_type: NodeType,
}

#[derive(Debug, Clone)]
pub struct Switch {
    pub is_cb: bool,
}

#[derive(Debug, Clone)]
pub struct Circuit {
    pub admittance: C32,
    pub line_charge: f32,
}

#[derive(Clone)]
pub struct PsEdge {
    pub index: EdgeIndex,
    pub name: String,
    pub u: U,
    pub data: EdgeData,
}

#[derive(Clone)]
pub enum EdgeData {
    Cir(Circuit),
    Sw(Switch),
}

impl PartialEq for EdgeData {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Cir(_), Self::Cir(_)) => true,
            (Self::Sw(_), Self::Sw(_)) => true,
            _ => false,
        }
    }
}

pub struct PowerSystem {
    pub g: Graph<PsNode, PsEdge>,
    pub start_u: Vec<U>,

    pub edges_names: HashMap<String, EdgeIndex>,
    pub slack_node_index: NodeIndex,
    pub sigma: SigAlg,
}

impl PowerSystem {
    pub fn from_files(path: &str) -> Self {
        let file_contents = file_parsing::parse_ps(path);

        let nodes: Vec<PsNode> = file_contents
            .nodes
            .iter()
            .map(|n| n.clone())
            .collect();

        let edges: Vec<FileEdge> = file_contents
            .edges
            .iter()
            .map(|e| e.clone())
            .collect();

        let slack_node_index = nodes.iter().find(|pn| pn.n_type == NodeType::Sk).map(|pn| pn.index).unwrap();

        let mut edges_names: HashMap<String, EdgeIndex> = HashMap::new();

        edges
            .iter()
            .filter(|ed| match &ed.edge.data {
                Cir(_) => false,
                Sw(sw) => sw.is_cb,
            })
            .enumerate()
            .for_each(|(num, ed)| {
                edges_names.insert(
                    ed.edge.data.get_type().to_string() + &(num + 1).to_string(),
                    ed.edge.index.clone()
                );
            });

        edges
            .iter()
            .filter(|ed| match &ed.edge.data {
                Cir(_) => false,
                Sw(sw) => !sw.is_cb,
            })
            .enumerate()
            .for_each(|(num, ed)| {
                edges_names.insert(
                    ed.edge.data.get_type().to_string() + &(num + 1).to_string(),
                    ed.edge.index,
                );
            });

        edges
            .iter()
            .filter(|ed| match ed.edge.data {
                Cir(_) => true,
                Sw(_) => false,
            })
            .enumerate()
            .for_each(|(num, ed)| {
                edges_names.insert(
                    ed.edge.data.get_type().to_string() + &(num + 1).to_string(),
                    ed.edge.index,
                );
            });

        let edge_is_quarantine = |index: EdgeIndex| match edges[index.0].edge.data {
            EdgeData::Cir(_) => false,
            EdgeData::Sw(_) => true,
        };

        let mut graph = Graph::empty_graph();
        nodes.iter().for_each(|pn| {
            graph.add_node(pn.clone());
        });

        edges.iter().for_each(|fe| {
            graph.add_edge(fe.edge.clone(), fe.fbus, fe.tbus);
        });

        let sigma = generate_sigma_alg(&graph, &edge_is_quarantine);

        PowerSystem {
            g: graph,
            start_u: file_contents.start_u,
            edges_names: edges_names,
            slack_node_index: slack_node_index,
            sigma,
        }
    }

    pub fn get_neighbors(&self, node_index: NodeIndex) -> &Vec<AdjacentInfo> {
        self.g.get_adjacency_info(node_index)
    }

    pub fn ps_node_iter(&self) -> Iter<'_, PsNode> {
        self.g.node_data.iter()
    }

    pub fn ps_edge_iter(&self) -> Iter<'_, PsEdge> {
        self.g.edge_data.iter()
    }

    pub fn edges(&self) -> Vec<Edge<'_, PsEdge>> {
        self.g.edges()
    }

    pub fn get_edge(&self, edge_index: EdgeIndex) -> Edge<'_, PsEdge> {
        self.g.get_edge(edge_index)
    }

    pub fn get_edge_by_name(&self, name: &String) -> Option<Edge<'_, PsEdge>> {
        let edge_index = self.edges_names.get(name);
        edge_index.map(|ind| self.g.get_edge(*ind))
    }

    pub fn create_sigma_alg<F>(&self, edge_is_quarantine: &F) -> SigAlg
    where
        F: Fn(EdgeIndex) -> bool,
    {
        generate_sigma_alg(&self.g, edge_is_quarantine)
    }

    pub fn node_count(&self) -> usize{
        self.g.get_node_count()
    }

    pub fn live_nodes(&self, u_vec: &Vec<U>) -> HashSet<NodeIndex> {
        plague_algo_pure(self.slack_node_index, &self.g, |ei| !self.g.edge_data[ei.0].conducts(&u_vec[ei.0])).iter().map(|ni| ni.clone()).collect()
    }
}

impl Display for Switch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Switch")
            .field("is_cb", &self.is_cb.to_string())
            .finish()
    }
}

impl Display for Circuit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Circuit")
            .field("admittance", &self.admittance.to_string())
            .field("line_charge", &self.line_charge)
            .finish()
    }
}

impl PsEdge {
    pub fn conducts(&self, u: &U) -> bool {
        match self.data {
            EdgeData::Cir(_) => true,
            EdgeData::Sw(_) => u != &U::Open,
        }
    }

    pub fn is_switch(self: &&Self) -> bool {
        match self.data {
            EdgeData::Sw(_) => true,
            EdgeData::Cir(_) => false,
        }
    }

    pub fn admittance(&self) -> C32 {
        match self.data {
            EdgeData::Cir(ref cir) => cir.admittance,
            EdgeData::Sw(_) => C32::new(0.0, 0.0),
        }
    }

    pub fn line_charge(&self) -> f32 {
        match self.data {
            EdgeData::Cir(ref cir) => cir.line_charge,
            EdgeData::Sw(_) => 0.0,
        }
    }

    pub fn quarantines_super_node(&self, u: &Option<&U>) -> bool {
        match self.data {
            EdgeData::Cir(_) => true,
            EdgeData::Sw(_) => u.unwrap() == &U::Open,
        }
    }
}

impl Debug for PsEdge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.data {
            EdgeData::Cir(c) => f
                .debug_struct("Edge")
                .field("type", &self.data.get_type().to_string())
                .field("admittance", &c.admittance.to_string())
                .field("line_c", &c.line_charge)
                .finish(),
            EdgeData::Sw(_) => f
                .debug_struct("Edge")
                .field("type", &self.data.get_type().to_string())
                .finish(),
        }
    }
}

impl Debug for PsNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PsNode")
            .field("num", &self.num)
            .field("load", &self.load.to_string())
            .field("gen", &self.gen.to_string())
            .finish()
    }
}

impl Debug for PowerSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PowerSystem")
            .field("nodes", &self.g.node_data)
            .field("edges", &self.g.edge_data)
            .field("start_u", &self.start_u)
            .finish()
    }
}

impl EdgeData {
    fn get_type(&self) -> &str {
        match &self {
            EdgeData::Cir(_) => "Cir",
            EdgeData::Sw(s) => {
                if s.is_cb {
                    "CB"
                } else {
                    "Dis"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BRB_FILE_PATH: &str = "./grids/BRB/";

    #[test]
    fn brb_gens() {
        let ps = PowerSystem::from_files(BRB_FILE_PATH);

        println!("BRB {:#?}", ps);

        let expected_gens = HashMap::from([(27, C32 { re: 45.0, im: 10.0 })]);

        ps.ps_node_iter().enumerate().for_each(|(i, node)| {
            assert_eq!(
                expected_gens
                    .get(&(i + 1))
                    .unwrap_or(&C32 { re: 0.0, im: 0.0 }),
                &node.gen
            );
        })
    }

    #[test]
    fn brb_loads() {
        let ps = PowerSystem::from_files(BRB_FILE_PATH);

        let expected_loads = HashMap::from([
            (5, C32 { re: 25.0, im: 5.0 }),
            (25, C32 { re: 25.0, im: 5.0 }),
            (
                26,
                C32 {
                    re: 250.0,
                    im: 80.0,
                },
            ),
        ]);

        ps.ps_node_iter().enumerate().for_each(|(i, node)| {
            assert_eq!(
                expected_loads
                    .get(&(i + 1))
                    .unwrap_or(&C32 { re: 0.0, im: 0.0 }),
                &node.load
            );
        })
    }
}

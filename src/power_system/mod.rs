use std::cmp;
use std::fmt;
use std::collections::HashMap;
use std::iter::zip;
use std::rc::Rc;
use std::slice::Iter;
use std::str::FromStr;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;

use crate::traits::C32;
use crate::power_system::EdgeData::Cir;
use crate::power_system::EdgeData::Sw;

use self::plague_algo::generate_sigma_alg;
use self::plague_algo::SimpleSigAlg;

mod file_parsing;
pub mod plague_algo;
pub mod power_flow_model;
pub mod outage;

type EdgeIndex = usize;
type NodeIndex = usize;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum U{
    Open,
    Closed,
    DontCare
}

impl fmt::Display for U {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl U {
    pub fn hamming_dist(target_u: &Vec<U>, actual_u: &Vec<U>) -> f32{
        zip(target_u.iter(), actual_u.iter())
        .map(|(t_u, a_u)| {
            match t_u {
                U::Open => if a_u == &U::Closed {1.0} else {0.0},
                U::Closed => if a_u == &U::Open {1.0} else {0.0},
                U::DontCare => 0.0,
            }
        })
        .sum()
    }

    pub fn not(&self) -> U{
        match self {
            U::Open => U::Closed,
            U::Closed => U::Open,
            U::DontCare => panic!(),
        }
    }
}

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone)]
pub enum NodeType {
	PQ, PV, Sk 
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
    pub index: usize,
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
pub struct PsNode{
    pub num: usize,
    pub index: NodeIndex,
    pub load: C32,
    pub gen: C32,
    pub system_v: f32,
    pub n_type: NodeType,
}

#[derive(Debug, Clone)]
pub struct Switch{
    pub is_cb: bool,
}

#[derive(Debug, Clone)]
pub struct Circuit {
    pub admittance: C32,
    pub line_charge: f32,
}

#[derive(Clone)]
pub struct Edge{
    pub index: EdgeIndex,
    pub name: String,
    pub fbus: Rc<PsNode>,
    pub tbus: Rc<PsNode>,
    pub data: EdgeData,
}

#[derive(Clone)]
pub enum EdgeData{
    Cir(Circuit),
    Sw(Switch),
}

impl PartialEq for EdgeData{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Cir(_), Self::Cir(_)) => true,
            (Self::Sw(_), Self::Sw(_)) => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct EdgePsNode{
    edge: Rc<Edge>,
    node: Rc<PsNode>
}

#[derive(Clone)]
pub struct PowerSystem {
    _nodes: Vec<PsNode>,
    _edges: Vec<Edge>,
    _switches: Vec<Edge>,

    pub nodes: Vec<Rc<PsNode>>,
    pub edges: Vec<Rc<Edge>>,

    pub start_u: Vec<U>,

    pub edges_names: HashMap<String, Rc<Edge>>,

    pub sigma: SimpleSigAlg,

    pub adjacent_node: HashMap<NodeIndex, Vec<EdgePsNode>>,
}

impl PowerSystem {
    pub fn from_files(path: &str) -> PowerSystem {
        let file_contents = file_parsing::parse_ps(path);

        let nodes: Vec<Rc<PsNode>> = file_contents.nodes.iter().map(|n| Rc::from(n.clone())).collect();
        let edges: Vec<Rc<Edge>> = file_contents.edges.iter().map(|e| Rc::from(e.clone())).collect();

        let adjacent_node = nodes.iter().map(|node| {
            
            let edge_map = edges.iter()
            .filter(|edge| edge.connected_to(node))
            .map(|edge| EdgePsNode{ edge: Rc::from(edge.clone()), node: edge.other_node(node).unwrap() } )
            .collect::<Vec<EdgePsNode>>();

            (node.index, edge_map)
        }).collect::<HashMap<NodeIndex, Vec<EdgePsNode>>>();

        let mut edges_names: HashMap<String, Rc<Edge>> = HashMap::new();

        edges.iter().filter(|ed| {
            match &ed.data {
                Cir(_) => false,
                Sw(sw) => sw.is_cb,
            }
        }).enumerate().for_each(|(num, ed)| {edges_names.insert(ed.data.get_type().to_string() + &(num + 1).to_string(), ed.clone());});

        edges.iter().filter(|ed| {
            match &ed.data {
                Cir(_) => false,
                Sw(sw) => !sw.is_cb,
            }
        }).enumerate().for_each(|(num, ed)| {edges_names.insert(ed.data.get_type().to_string() + &(num + 1).to_string(), ed.clone());});

        edges.iter().filter(|ed| {
            match ed.data {
                Cir(_) => true,
                Sw(_) => false,
            }
        }).enumerate().for_each(|(num, ed)| {edges_names.insert( ed.data.get_type().to_string() + &(num + 1).to_string(), ed.clone());});


        let switches = file_contents.edges.iter().filter(Edge::is_switch).map(Edge::clone).collect::<Vec<Edge>>();


        let edge_is_quarantine = |index: EdgeIndex| match edges.get(index).unwrap().data {
            EdgeData::Cir(_) => false,
            EdgeData::Sw(_) => true,
        };

        let sigma = generate_sigma_alg(&adjacent_node, &nodes, &edge_is_quarantine);

        PowerSystem { 
            _nodes: file_contents.nodes, 
            _edges: file_contents.edges, 
            _switches: switches, 
            start_u: file_contents.start_u, 
            nodes: nodes,
            edges: edges,
            adjacent_node: adjacent_node,
            edges_names: edges_names,
            sigma,
        }
    }

    fn get_neighbors(&self, node_index: &usize) -> &Vec<EdgePsNode> {
        self.adjacent_node.get(node_index).unwrap()
    }

    fn nodes_iter(&self) -> Iter<'_, PsNode> {
        self._nodes.iter()
    } 

    fn switch_iter(&self) -> Iter<'_, Edge> {
        self._switches.iter()
    }

    fn edges_iter(&self) -> Iter<'_, Edge> {
        self._edges.iter()
    }

    pub fn get_edge_by_name(&self, name: &String) -> Option<&Rc<Edge>> {
        return self.edges_names.get(name);
    }
}


impl Display for Switch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Switch").field("is_cb", &self.is_cb.to_string()).finish()
    }
}

impl Display for Circuit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Circuit").field("admittance", &self.admittance.to_string()).field("line_charge", &self.line_charge).finish()
    }
}

impl Edge {
    pub fn connected_to(&self, node: &PsNode) -> bool {
         self.tbus.as_ref() == node || self.fbus.as_ref() == node 
    }

    pub fn other_node(&self, node: &PsNode) -> Option<Rc<PsNode>>{
        if self.tbus.as_ref() == node {
            Some(self.fbus.clone())
        } else if self.fbus.as_ref() == node {
            Some(self.tbus.clone())
        } else {
            None
        }
    }

    pub fn conducts(&self, u: &U) -> bool {
        match self.data {
            EdgeData::Cir(_) => true,
            EdgeData::Sw(_) => u != &U::Open
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
            EdgeData::Sw(ref sw) => C32::new(0.0, 0.0),
        }
    }

    pub fn quarantines_super_node(&self, u: &Option<&U>) -> bool {
        match self.data {
            EdgeData::Cir(_) => true,
            EdgeData::Sw(_) => u.unwrap() == &U::Open
        } 
    }
}

impl Debug for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.data {
            EdgeData::Cir(c) => f.debug_struct("Edge").field("type", &self.data.get_type().to_string()).field("tbus", &self.tbus.num).field("fbus", &self.fbus.num).field("admittance", &c.admittance.to_string()).field("line_c", &c.line_charge).finish(),
            EdgeData::Sw(_) => f.debug_struct("Edge").field("type", &self.data.get_type().to_string()).field("tbus", &self.tbus.num).field("fbus", &self.fbus.num).finish(),
        }
    }
}

impl Debug for PsNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PsNode").field("num", &self.num).field("load", &self.load.to_string()).field("gen", &self.gen.to_string()).finish()
    }
}

impl Debug for PowerSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PowerSystem").field("nodes", &self._nodes).field("edges", &self._edges).field("start_u", &self.start_u).finish()
    }
}

impl EdgeData {  
    fn get_type(&self) -> &str{
        match &self {
            EdgeData::Cir(_) => "Cir",
            EdgeData::Sw(s) => {
                if s.is_cb {
                    "CB"
                } else {
                    "Dis"
                }
            },
        }
    }
}

impl Debug for EdgePsNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EdgePsNode").field("node", &self.node.num).field("edge", &self.edge).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BRB_FILE_PATH: &str = "./grids/BRB/"; 

    #[test]
    fn brb_gens(){
        let ps = PowerSystem::from_files(BRB_FILE_PATH);

        println!("BRB {:#?}", ps);

        let expected_gens = HashMap::from([(27, C32{re: 45.0, im: 10.0})]);

        ps._nodes.iter().enumerate().for_each(|(i, node)| {
            assert_eq!(expected_gens.get(&(i + 1)).unwrap_or(&C32{re:0.0, im:0.0}), &node.gen);
        })
    }

    #[test]
    fn brb_loads(){
        let ps = PowerSystem::from_files(BRB_FILE_PATH);

        let expected_loads = HashMap::from([
            (5, C32{re: 25.0, im: 5.0}),
            (25, C32{re: 25.0, im: 5.0}),
            (26, C32{re: 250.0, im: 80.0}),
            ]);

            ps._nodes.iter().enumerate().for_each(|(i, node)| {
                assert_eq!(expected_loads.get(&(i + 1)).unwrap_or(&C32{re:0.0, im:0.0}), &node.load);
            })
    }
}

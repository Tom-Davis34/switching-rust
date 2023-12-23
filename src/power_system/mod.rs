use std::fmt;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;
use std::fmt::Debug;
use std::fmt::Display;

use crate::traits::C32;
mod file_parsing;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum U{
    Open,
    Closed,
    DontCare
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NodeType {
	GND, PQ, PV, Sk 
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError;

#[derive(PartialEq, Clone)]
pub struct PsNode{
    pub num: usize,
    pub load: C32,
    pub gen: C32,
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
    pub fbus: Rc<PsNode>,
    pub tbus: Rc<PsNode>,
    pub data: EdgeData,
}

#[derive(Clone)]
pub enum EdgeData{
    Cir(Circuit),
    Sw(Switch),
}

#[derive(Clone)]
pub struct EdgePsNode{
    edge: Edge,
    node: Rc<PsNode>
}

#[derive(Clone)]
pub struct PowerSystem {
    pub nodes: Vec<PsNode>,
    pub edges: Vec<Edge>,

    pub start_u: Vec<U>,

    pub adjacent_node: HashMap<usize, Vec<EdgePsNode>>,
}

impl PowerSystem {
    fn from_files(path: &str) -> PowerSystem {
        let file_contents = file_parsing::parse_ps(path);

        let adjacent_node = file_contents.nodes.iter().map(|node| {
            
            let edge_map = file_contents.edges.iter()
            .filter(|edge| edge.connected_to(node))
            .map(|edge| EdgePsNode{ edge: edge.clone(), node: edge.other_node(node).unwrap() } )
            .collect::<Vec<EdgePsNode>>();

            (node.num, edge_map)
        }).collect::<HashMap<usize, Vec<EdgePsNode>>>();


        PowerSystem { 
            nodes: file_contents.nodes, 
            edges: file_contents.edges, 
            start_u: file_contents.start_u, 
            adjacent_node: adjacent_node,
        }
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
        f.debug_struct("PowerSystem").field("nodes", &self.nodes).field("edges", &self.edges).field("start_u", &self.start_u).finish()
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

        let expected_gens = HashMap::from([(27, C32{re: 45.0, im: 10.0})]);

        ps.nodes.iter().enumerate().for_each(|(i, node)| {
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

            ps.nodes.iter().enumerate().for_each(|(i, node)| {
                assert_eq!(expected_loads.get(&(i + 1)).unwrap_or(&C32{re:0.0, im:0.0}), &node.load);
            })
    }
}

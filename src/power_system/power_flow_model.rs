use std::{
    iter::{self, zip},
    rc::{Rc, Weak},
};

use crate::traits::C32;

use super::{
    plague_algo::{plague_algo, SigAlg, SimpleSigAlg},
    DeltaU, Edge, EdgeData, EdgeIndex, NodeType, PowerSystem, PsNode, U,
};

#[derive(Debug, PartialEq, Clone)]
pub struct VectorIndex(usize);

#[derive(Debug, Clone)]
pub struct PfNode {
    pub index: VectorIndex,
    pub pq: C32,
    pub n_type: NodeType,
    pub is_load: bool,
    pub system_v: f32,
    pub unit_v: Option<C32>,
    pub nodes: Vec<Rc<PsNode>>,
    pub edges: Vec<Weak<PfEdge>>,
}

impl PartialEq for PfNode {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

#[derive(Debug, Clone)]
pub struct PfEdge {
    pub index: VectorIndex,
    pub edge: Rc<Edge>,
    pub tnode: Weak<PfNode>,
    pub fnode: Weak<PfNode>,
    pub unit_current: Option<C32>,
}

impl PartialEq for PfEdge {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

#[derive(Debug, Clone)]
pub struct PfGraph {
    pub nodes: Vec<Rc<PfNode>>,
    ps_node_to_pf_node: Vec<Rc<PfNode>>,
    pub edges: Vec<Rc<PfEdge>>,
    ps_edge_to_pf_edge: Vec<Option<Rc<PfEdge>>>,
}

pub fn generate_pf_graph(ps: &PowerSystem, u: &Vec<U>) -> PfGraph {
    let edge_u = zip(ps.edges.iter(), u.iter())
        .map(|tu| (tu.0.clone(), tu.1.clone()))
        .collect::<Vec<(Rc<Edge>, U)>>();

    let edge_is_quarantine = |index: EdgeIndex| {
        let eu = edge_u.get(index).unwrap();

        match eu.0.data {
            EdgeData::Cir(_) => true,
            EdgeData::Sw(_) => eu.1 == U::Open,
        }
    };

    let sig_alg = super::plague_algo::create_sigma_alg(&ps, &edge_is_quarantine);

    let pf_nodes_temp = generate_pf_nodes(&sig_alg);

    let pf_edges = generate_pf_edges(&ps, &sig_alg, &pf_nodes_temp);

    let pf_nodes: Vec<Rc<PfNode>> = pf_nodes_temp
        .into_iter()
        .map(|pf_node| {
            let edges = pf_edges
                .iter()
                .filter(|e| {
                    e.tnode.upgrade().unwrap() == pf_node || e.fnode.upgrade().unwrap() == pf_node
                })
                .map(Rc::downgrade)
                .collect::<Vec<Weak<PfEdge>>>();

            Rc::new(PfNode {
                index: pf_node.index.clone(),
                pq: pf_node.pq.clone(),
                is_load: pf_node.is_load,
                system_v: pf_node.system_v,
                unit_v: pf_node.unit_v,
                nodes: pf_node.nodes.iter().map(|n| n.clone()).collect::<Vec<Rc<PsNode>>>(),
                edges: edges,
                n_type: pf_node.nodes.iter().map(|n| n.n_type.clone()).max().unwrap(),
            })
        })
        .collect();

    let mut ps_edge_to_pf_edge = ps
        .nodes
        .iter()
        .map(|n| None)
        .collect::<Vec<Option<Rc<PfEdge>>>>();

    for ele in pf_edges.iter() {
        ps_edge_to_pf_edge[ele.edge.index] = Some(ele.clone());
    }

    let ps_node_to_pf_node = ps
        .nodes
        .iter()
        .enumerate()
        .map(|(_i, node)| pf_nodes[sig_alg.get_basis(node).index].clone())
        .collect::<Vec<Rc<PfNode>>>();

    return PfGraph {
        nodes: pf_nodes,
        ps_node_to_pf_node,
        edges: pf_edges,
        ps_edge_to_pf_edge,
    };
}

fn generate_pf_edges(
    ps: &PowerSystem,
    sig_alg: &SimpleSigAlg,
    pf_nodes: &Vec<Rc<PfNode>>,
) -> Vec<Rc<PfEdge>> {
    let ps_node_to_pf_node = ps
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| pf_nodes[sig_alg.get_basis(node).index].clone())
        .collect::<Vec<Rc<PfNode>>>();

    let edges: Vec<Rc<PfEdge>> = ps
        .edges
        .iter()
        .filter(|edge| {
            ps_node_to_pf_node[edge.tbus.index].index != ps_node_to_pf_node[edge.fbus.index].index
        })
        .enumerate()
        .map(|(index, edge)| {
            let pf_tnode = &ps_node_to_pf_node[edge.tbus.index];
            let pf_fnode = &ps_node_to_pf_node[edge.fbus.index];

            Rc::new(PfEdge {
                index: VectorIndex(index),
                edge: edge.clone(),
                tnode: Rc::downgrade(&pf_tnode),
                fnode: Rc::downgrade(&pf_fnode),
                unit_current: None,
            })
        })
        .collect();

    return edges;
}

fn generate_pf_nodes(sig_alg: &SimpleSigAlg) -> Vec<Rc<PfNode>> {
    sig_alg
        .basis
        .iter()
        .enumerate()
        .map(|(index, basis)| {
            let pq = basis.nodes.iter().map(|n| n.gen - n.load).sum();
            let is_load = basis.nodes.iter().any(|n| n.load != C32::new(0.0, 0.0));
            let system_v = basis.nodes.first().unwrap().system_v;

            Rc::new(PfNode {
                index: VectorIndex(index),
                pq,
                n_type: basis.nodes.iter().map(|n| n.n_type.clone()).max().unwrap(),
                is_load,
                system_v,
                unit_v: None,
                nodes: basis
                    .nodes
                    .iter()
                    .map(|n| n.clone())
                    .collect::<Vec<Rc<PsNode>>>(),
                edges: vec![],
            })
        })
        .collect::<Vec<Rc<PfNode>>>()
}


mod tests {
    use crate::power_system::PowerSystem;

    use super::*;

    const BRB_FILE_PATH: &str = "./grids/BRB/";

    #[test]
    fn sigma_alg() {
        let ps = PowerSystem::from_files(BRB_FILE_PATH);

                
    }
}
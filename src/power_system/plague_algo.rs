use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Display,
    iter::{self, zip},
    rc::Rc,
};

use super::{
    Circuit, DeltaU, Edge, EdgeData, EdgeIndex, EdgePsNode, NodeIndex, PowerSystem, PsNode, U,
};

pub trait SigBasis {
    fn nodes(&self) -> &Vec<Rc<PsNode>>;
}

pub trait SigAlg<B: SigBasis> {
    fn get_basis(&self, node: &PsNode) -> &B;
    fn basis_vec(&self) -> &Vec<Rc<B>>;
}

#[derive(Debug, Clone)]
pub struct SimpleSigAlg {
    pub to_basis: Vec<Rc<SimpleBasisEle>>,
    pub basis: Vec<Rc<SimpleBasisEle>>,
}

impl SigAlg<SimpleBasisEle> for SimpleSigAlg {
    fn get_basis(&self, node: &PsNode) -> &SimpleBasisEle {
        &self.to_basis[node.index]
    }

    fn basis_vec(&self) -> &Vec<Rc<SimpleBasisEle>> {
        &self.basis
    }
}

#[derive(Debug, Clone)]
pub struct SimpleBasisEle {
    pub index: usize,
    pub nodes: Vec<Rc<PsNode>>,
}

impl SigBasis for SimpleBasisEle {
    fn nodes(&self) -> &Vec<Rc<PsNode>> {
        &self.nodes
    }
}

struct PlagueAlgoState<'a> {
    stk: Vec<NodeIndex>,
    ret_val: Vec<NodeIndex>,
    infected: &'a mut Vec<bool>,
}

impl<'a> PlagueAlgoState<'a> {
    fn new(infected: &'a mut Vec<bool>) -> PlagueAlgoState<'a> {
        return PlagueAlgoState {
            stk: vec![],
            ret_val: vec![],
            infected: infected,
        };
    }

    fn spread_infection(&mut self, node_index: NodeIndex) {
        if self.infected[node_index] {
            return;
        } else {
            self.stk.push(node_index);
            self.infected[node_index] = true;
            self.ret_val.push(node_index);
        }
    }
}

pub fn plague_algo<'a, F>(
    start_node_index: NodeIndex,
    adjacent_node: &HashMap<usize, Vec<EdgePsNode>>,
    infected: &'a mut Vec<bool>,
    edge_is_quarantine: F,
) -> Vec<NodeIndex>
where
    F: Fn(EdgeIndex) -> bool,
{
    let mut plague_state = PlagueAlgoState::<'a>::new(infected);
    plague_state.spread_infection(start_node_index);

    while !plague_state.stk.is_empty() {
        let current_node = plague_state.stk.pop().unwrap();

        let neighbors = adjacent_node.get(&current_node).unwrap();

        neighbors.iter().for_each(|neighbour: &super::EdgePsNode| {
            if !edge_is_quarantine(neighbour.edge.index) {
                plague_state.spread_infection(neighbour.node.index);
            }
        });
    }

    // println!("ret_val, {:?}", ret_val);
    return plague_state.ret_val;
}

pub fn create_sigma_alg<F>(ps: &PowerSystem, edge_is_quarantine: &F) -> SimpleSigAlg
where
    F: Fn(EdgeIndex) -> bool,
{
    generate_sigma_alg(&ps.adjacent_node, &ps.nodes, edge_is_quarantine)
}

pub fn generate_sigma_alg<F>(
    adjacent_node: &HashMap<usize, Vec<EdgePsNode>>,
    nodes: &Vec<Rc<PsNode>>,
    edge_is_quarantine: &F,
) -> SimpleSigAlg
where
    F: Fn(EdgeIndex) -> bool,
{
    let mut basis = vec![];
    let mut infected = iter::repeat(false).take(nodes.len()).collect::<Vec<bool>>();

    nodes.iter().enumerate().for_each(|(node_index, _node)| {
        if infected[node_index] {
            return;
        }

        let group = plague_algo(
            node_index,
            &adjacent_node,
            &mut infected,
            edge_is_quarantine,
        );
        basis.push(group);
    });

    let basis_eles = basis
        .iter()
        .enumerate()
        .map(|group| {
            return Rc::new(SimpleBasisEle {
                index: group.0,
                nodes: group
                    .1
                    .iter()
                    .map(|node_index| nodes[*node_index].clone())
                    .collect::<Vec<Rc<PsNode>>>(),
            });
        })
        .collect::<Vec<Rc<SimpleBasisEle>>>();

    let mut to_b = iter::repeat(None)
        .take(nodes.len())
        .collect::<Vec<Option<Rc<SimpleBasisEle>>>>();
    basis_eles.iter().for_each(|bi| {
        bi.nodes().iter().for_each(|node| {
            to_b[node.index].replace(bi.clone());
        })
    });

    SimpleSigAlg {
        to_basis: to_b.into_iter().map(|val| val.unwrap()).collect(),
        basis: basis_eles,
    }
}

mod tests {
    use crate::power_system::PowerSystem;

    use super::*;

    const BRB_FILE_PATH: &str = "./grids/BRB/";

    #[test]
    fn sigma_alg() {
        let ps = PowerSystem::from_files(BRB_FILE_PATH);

        let edge_is_quarantine = |index: EdgeIndex| match ps.edges.get(index).unwrap().data {
            EdgeData::Cir(_) => false,
            EdgeData::Sw(_) => true,
        };

        let sig: SimpleSigAlg =
            generate_sigma_alg(&ps.adjacent_node,  &ps.nodes, &edge_is_quarantine);

        // println!("{:#?}", sig);

        assert_eq!(sig.to_basis.len(), ps.nodes.len());
        assert_eq!(
            sig.basis.iter().flat_map(|b| b.nodes().iter()).count(),
            ps.nodes.len()
        );

        ps.nodes_iter().for_each(|n| {
            assert!(sig.to_basis[n.index]
                .nodes()
                .iter()
                .find(|n2| n2.index == n.index)
                .is_some());
        });

        let basis26 = sig
            .basis
            .iter()
            .find(|b| b.nodes().iter().find(|n| n.num == 26).is_some())
            .unwrap();

        assert!(basis26.nodes().iter().find(|n| n.num == 26).is_some());
        assert!(basis26.nodes().iter().find(|n| n.num == 28).is_some());
        assert!(basis26.nodes().iter().find(|n| n.num == 20).is_some());
        assert!(basis26.nodes().iter().find(|n| n.num == 30).is_some());
        assert!(basis26.nodes().iter().find(|n| n.num == 2).is_some());
        assert!(basis26.nodes().len() == 5);
    }

    #[test]
    fn all_closed() {
        let ps = PowerSystem::from_files(BRB_FILE_PATH);
        let edge_is_quarantine = |index: EdgeIndex| match ps.edges.get(index).unwrap().data {
            EdgeData::Cir(_) => false,
            EdgeData::Sw(_) => false,
        };

        let res = create_sigma_alg(&ps, &edge_is_quarantine);

        println!("{:?}", res);

        assert_eq!(res.basis.len(), 1);
    }

    #[test]
    fn all_closed_cir_connected() {
        let ps = PowerSystem::from_files(BRB_FILE_PATH);
        let edge_is_quarantine = |index: EdgeIndex| match ps.edges.get(index).unwrap().data {
            EdgeData::Cir(_) => true,
            EdgeData::Sw(_) => false,
        };

        let res = create_sigma_alg(&ps, &edge_is_quarantine);

        println!("{:?}", res);

        assert_eq!(res.basis.len(), 6);
    }

    #[test]
    fn all_open() {
        let ps = PowerSystem::from_files(BRB_FILE_PATH);

        let edge_is_quarantine = |index: EdgeIndex| match ps.edges.get(index).unwrap().data {
            EdgeData::Cir(_) => true,
            EdgeData::Sw(_) => true,
        };

        let res = create_sigma_alg(&ps, &edge_is_quarantine);

        println!("{:?}", res);

        assert_eq!(res.basis.len(), ps.nodes.len());
    }
}

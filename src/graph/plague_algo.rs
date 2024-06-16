use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Display,
    iter::{self, zip},
    rc::Rc,
};

use crate::graph::Graph;

use super::*;

#[derive(Debug, Clone)]
pub struct SigBasis {
    pub index: usize,
    pub nodes: Vec<NodeIndex>,
}
#[derive(Debug, Clone)]
pub struct SigAlg {
    pub to_basis: Vec<Rc<SigBasis>>,
    pub basis: Vec<Rc<SigBasis>>,
}

impl SigAlg {
    fn get_basis(&self, node_index: NodeIndex) -> &SigBasis {
        &self.to_basis[node_index.0]
    }

    fn basis_vec(&self) -> &Vec<Rc<SigBasis>> {
        &self.basis
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
        if self.infected[node_index.0] {
            return;
        } else {
            self.stk.push(node_index);
            self.infected[node_index.0] = true;
            self.ret_val.push(node_index);
        }
    }
}

pub fn plague_algo_pure<'a, F, N, E>(
    start_node_index: NodeIndex,
    graph: &Graph<N, E>,
    edge_is_quarantine: F,
) -> Vec<NodeIndex>
where
    F: Fn(EdgeIndex) -> bool,
{
    let mut infected = iter::repeat(false)
        .take(graph.get_node_count())
        .collect::<Vec<bool>>();
    return plague_algo(start_node_index, graph, &mut infected, edge_is_quarantine);
}

pub fn plague_algo<'a, F, N, E>(
    start_node_index: NodeIndex,
    graph: &Graph<N, E>,
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

        let neighbors = graph.get_adjacency_info(current_node);

        neighbors.iter().for_each(|neighbour| {
            if !edge_is_quarantine(neighbour.edge_index) {
                plague_state.spread_infection(neighbour.node_index);
            }
        });
    }

    // println!("ret_val, {:?}", ret_val);
    return plague_state.ret_val;
}

pub fn generate_sigma_alg<F, N, E>(graph: &Graph<N, E>, edge_is_quarantine: &F) -> SigAlg
where
    F: Fn(EdgeIndex) -> bool,
{
    let mut basis = vec![];
    let mut infected = iter::repeat(false)
        .take(graph.get_node_count())
        .collect::<Vec<bool>>();

    graph.nodes.iter().for_each(|node| {
        if infected[node.index.0] {
            return;
        }

        let group = plague_algo(node.index, graph, &mut infected, edge_is_quarantine);
        basis.push(group);
    });

    let basis_eles = basis
        .iter()
        .enumerate()
        .map(|group| {
            return Rc::new(SigBasis {
                index: group.0,
                nodes: group
                    .1
                    .iter()
                    .map(|node_index| node_index.clone())
                    .collect::<Vec<NodeIndex>>(),
            });
        })
        .collect::<Vec<Rc<SigBasis>>>();

    let mut to_b = iter::repeat(None)
        .take(graph.nodes.len())
        .collect::<Vec<Option<Rc<SigBasis>>>>();

    basis_eles.iter().for_each(|bi| {
        bi.nodes.iter().for_each(|node| {
            to_b[node.0].replace(bi.clone());
        })
    });

    SigAlg {
        to_basis: to_b.into_iter().map(|val| val.unwrap()).collect(),
        basis: basis_eles,
    }
}

mod tests {
    use crate::power_system::{EdgeData, PowerSystem};

    use super::*;

    const BRB_FILE_PATH: &str = "./grids/BRB/";

    #[test]
    fn sigma_alg() {
        let ps = PowerSystem::from_files(BRB_FILE_PATH);

        let edge_is_quarantine = |index: EdgeIndex| match ps.g.get_edge(index).data.data {
            EdgeData::Cir(_) => false,
            EdgeData::Sw(_) => true,
        };

        let sig: SigAlg = ps.create_sigma_alg(&edge_is_quarantine);

        // println!("{:#?}", sig);

        assert_eq!(sig.to_basis.len(), ps.g.nodes.len());
        assert_eq!(
            sig.basis.iter().flat_map(|b| b.nodes.iter()).count(),
            ps.g.nodes.len()
        );

        ps.ps_node_iter().for_each(|n| {
            assert!(sig.to_basis[n.index.0]
                .nodes
                .iter()
                .find(|n2| n2 == &&n.index)
                .is_some());
        });

        let basis26 = sig
            .basis
            .iter()
            .find(|b| b.nodes.iter().find(|n| n == &&NodeIndex(25)).is_some())
            .unwrap();

        assert!(basis26
            .nodes
            .iter()
            .find(|n| n == &&NodeIndex(25))
            .is_some());
        assert!(basis26
            .nodes
            .iter()
            .find(|n| n == &&NodeIndex(27))
            .is_some());
        assert!(basis26
            .nodes
            .iter()
            .find(|n| n == &&NodeIndex(19))
            .is_some());
        assert!(basis26
            .nodes
            .iter()
            .find(|n| n == &&NodeIndex(29))
            .is_some());
        assert!(basis26.nodes.iter().find(|n| n == &&NodeIndex(1)).is_some());
        assert!(basis26.nodes.len() == 5);
    }

    #[test]
    fn all_closed() {
        let ps = PowerSystem::from_files(BRB_FILE_PATH);
        let edge_is_quarantine = |index: EdgeIndex| match ps.g.get_edge(index).data.data {
            EdgeData::Cir(_) => false,
            EdgeData::Sw(_) => false,
        };

        let res = ps.create_sigma_alg(&edge_is_quarantine);

        println!("{:?}", res);

        assert_eq!(res.basis.len(), 1);
    }

    #[test]
    fn all_closed_cir_connected() {
        let ps = PowerSystem::from_files(BRB_FILE_PATH);
        let edge_is_quarantine = |index: EdgeIndex| match ps.g.get_edge(index).data.data {
            EdgeData::Cir(_) => true,
            EdgeData::Sw(_) => false,
        };

        let res = ps.create_sigma_alg(&edge_is_quarantine);

        println!("{:?}", res);

        assert_eq!(res.basis.len(), 6);
    }

    #[test]
    fn all_open() {
        let ps = PowerSystem::from_files(BRB_FILE_PATH);

        let edge_is_quarantine = |index: EdgeIndex| match ps.g.get_edge(index).data.data {
            EdgeData::Cir(_) => true,
            EdgeData::Sw(_) => true,
        };

        let res = ps.create_sigma_alg(&edge_is_quarantine);

        println!("{:?}", res);

        assert_eq!(res.basis.len(), ps.g.node_data.len());
    }
}

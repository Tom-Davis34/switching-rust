use std::{
    collections::{HashMap, HashSet},
    iter::{self, zip},
    rc::Rc,
};

use super::{
    BasisEle, Circuit, DeltaU, Edge, EdgeData, EdgeIndex, EdgePsNode, NodeIndex, Outage,
    PowerSystem, PsNode, SigmAlg, U,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PowerFlowNode {}

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

pub fn generate_sigma_alg(
    adjacent_node: &HashMap<usize, Vec<EdgePsNode>>,
    edges: &Vec<Rc<Edge>>,
    nodes: &Vec<Rc<PsNode>>,
) -> SigmAlg {
    let mut basis = vec![];
    let mut infected = iter::repeat(false).take(nodes.len()).collect::<Vec<bool>>();

    let edge_is_quarantine = |index: EdgeIndex| match edges.get(index).unwrap().data {
        EdgeData::Cir(_) => false,
        EdgeData::Sw(_) => true,
    };

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
            return Rc::new(BasisEle {
                id: group.0,
                nodes: group
                    .1
                    .iter()
                    .map(|node_index| nodes[*node_index].clone())
                    .collect::<Vec<Rc<PsNode>>>(),
            });
        })
        .collect::<Vec<Rc<BasisEle>>>();

    let mut to_b = iter::repeat(None)
        .take(nodes.len())
        .collect::<Vec<Option<Rc<BasisEle>>>>();
    basis_eles.iter().for_each(|bi| {
        bi.nodes.iter().for_each(|node| {
            to_b[node.index].replace(bi.clone());
        })
    });

    SigmAlg {
        to_basis: to_b.into_iter().map(|val| val.unwrap()).collect(),
        basis: basis_eles,
    }
}

pub fn generate_outage(ps: &PowerSystem, edge_names: Vec<String>) -> Outage {
    let edge_indices = edge_names
        .iter()
        .map(|en| ps.get_edge_by_name(en).unwrap().clone())
        .collect::<Vec<EdgeIndex>>();

    let edges = edge_indices.iter().map(|ei| ps.edges.get(*ei).unwrap().clone()).collect::<Vec<Rc<Edge>>>();

    let basis_eles_dups = edges
    .iter()
    .flat_map(|f| vec![f.tbus.index, f.fbus.index].iter())
    .map(|i| ps.sigma.to_basis.get(*i).unwrap().id)
    .collect::<HashSet<usize>>();

    let basis_eles = basis_eles_dups.iter().map(|i| ps.sigma.basis.get(*i).unwrap().clone()).collect::<Vec<Rc<BasisEle>>>();
    let outage_nodes = basis_eles.iter().flat_map(|be| be.nodes.iter().map(|n| n.index)).collect::<Vec<NodeIndex>>();

    let boundary = ps.edges.iter().filter(|e| outage_nodes.contains(&e.fbus.index) != outage_nodes.contains(&e.tbus.index)).map(|e| e.clone()).collect::<Vec<Rc<Edge>>>();

    let delta_u = boundary.iter().filter(|e| {
        match ps.start_u.get(e.index).unwrap(){
            U::Open => false,
            U::Closed => true,
            U::DontCare => true,
        }
    }).map(|e|{
        DeltaU { index: e.index, new_u: U::Closed }
    }).collect::<Vec<DeltaU>>();
    
    Outage {
        in_outage: ps.nodes_iter().map(|i| outage_nodes.contains(i)).collect(),
        basis: basis_eles,
        boundary: boundary,
        delta_u: delta_u,
    }
}

pub fn generate_super_node_mapping(ps: &PowerSystem, delta_u: &Vec<DeltaU>) -> Vec<Vec<usize>> {
    let mut u = ps.start_u.clone();
    let mut ret_val = Vec::new();

    delta_u
        .iter()
        .for_each(|du: &DeltaU| u[du.index] = du.new_u);

    let mut infected = iter::repeat(false)
        .take(ps.nodes.len())
        .collect::<Vec<bool>>();

    let edge_is_quarantine = |index: EdgeIndex| {
        // println!("index {:?}", index);
        // println!("edge {:?}", ps.edges.get(index).unwrap());
        // println!("quarantines_super_node {:?}", ps.edges.get(index).unwrap().quarantines_super_node(&u.get(index)));
        ps.edges
            .get(index)
            .unwrap()
            .quarantines_super_node(&u.get(index))
    };

    ps.nodes_iter().enumerate().for_each(|(node_index, _node)| {
        if infected[node_index] {
            return;
        }

        let group = plague_algo(
            node_index,
            &ps.adjacent_node,
            &mut infected,
            edge_is_quarantine,
        );
        ret_val.push(group);
    });

    return ret_val;
}

mod tests {
    use crate::power_system::EdgeData;

    use super::*;

    const BRB_FILE_PATH: &str = "./grids/BRB/";

    #[test]
    fn all_closed() {
        let ps = PowerSystem::from_files(BRB_FILE_PATH);

        let delta_u = ps
            .edges_iter()
            .enumerate()
            .filter(|e| match e.1.data {
                EdgeData::Cir(_) => false,
                EdgeData::Sw(_) => true,
            })
            .map(|e| {
                return DeltaU {
                    index: e.0,
                    new_u: U::Closed,
                };
            })
            .collect::<Vec<DeltaU>>();

        let res = generate_super_node_mapping(&ps, &delta_u);

        println!("{:?}", res);

        assert_eq!(res.len(), 6);
    }

    #[test]
    fn all_open() {
        let ps = PowerSystem::from_files(BRB_FILE_PATH);

        let delta_u = ps
            .edges_iter()
            .enumerate()
            .filter(|e| match e.1.data {
                EdgeData::Cir(_) => false,
                EdgeData::Sw(_) => true,
            })
            .map(|e| {
                return DeltaU {
                    index: e.0,
                    new_u: U::Open,
                };
            })
            .collect::<Vec<DeltaU>>();

        let res = generate_super_node_mapping(&ps, &delta_u);

        println!("{:?}", res);

        assert_eq!(res.len(), ps.nodes.len());
    }

    #[test]
    fn sigma_alg() {
        let ps = PowerSystem::from_files(BRB_FILE_PATH);

        let sig: SigmAlg = generate_sigma_alg(&ps.adjacent_node, &ps.edges, &ps.nodes);

        // println!("{:#?}", sig);

        assert_eq!(sig.to_basis.len(), ps.nodes.len());
        assert_eq!(
            sig.basis.iter().flat_map(|b| b.nodes.iter()).count(),
            ps.nodes.len()
        );
    }
}

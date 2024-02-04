use std::{collections::{HashSet, HashMap}, iter::{self, zip}, rc::Rc};

use super::{U, PowerSystem, Edge, NodeIndex, EdgeIndex, Circuit, EdgeData, DeltaU, Outage, SigmAlg, EdgePsNode, PsNode, BasisEle};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PowerFlowNode {

}

fn spread_infection<F>( node_index: NodeIndex, adjacent_node: &HashMap<usize, Vec<EdgePsNode>>, visited_nodes: &mut Vec<bool>, stk: &mut Vec<NodeIndex>, edge_is_quarantine: F) where F: Fn(EdgeIndex) -> bool{
    let neighbors = adjacent_node.get(&node_index).unwrap();
    
    // println!("neighbors, {:?}", neighbors);
    
    neighbors.iter().for_each(|neighbour: &super::EdgePsNode| {
        if !edge_is_quarantine(neighbour.edge.index) && !visited_nodes[neighbour.node.index] {
            visited_nodes[neighbour.node.index] = true;
            stk.push(neighbour.node.index.clone())
        } else {
            // println!("!edge_is_quarantine(neighbour.edge.index) {:?}", !edge_is_quarantine(neighbour.edge.index));
            // println!("!visited_nodes[neighbour.node.index] {:?}", !visited_nodes[neighbour.node.index]);
        }
    });

    // println!("stk, {:?}", stk);
}

pub fn plague_algo<F>(start_node_index: NodeIndex, adjacent_node: &HashMap<usize, Vec<EdgePsNode>>, visited_nodes: &mut Vec<bool>, edge_is_quarantine: F) -> Vec<NodeIndex>
where F: Fn(EdgeIndex) -> bool{    
    let mut stk = vec![start_node_index];
    let mut ret_val = vec![];

    while !stk.is_empty() {
        let current_node = stk.pop().unwrap();
        ret_val.push(current_node);

        spread_infection(current_node, adjacent_node, visited_nodes, &mut stk, &edge_is_quarantine);
        // println!("stk, {:?}", stk);
    }

    // println!("ret_val, {:?}", ret_val);
    ret_val
}

pub fn generate_sigma_alg(adjacent_node: &HashMap<usize, Vec<EdgePsNode>>, edges: &Vec<Rc<Edge>>, nodes: &Vec<Rc<PsNode>>) -> SigmAlg {

    let mut basis = vec![];
    let mut visited_nodes = iter::repeat(false).take(nodes.len()).collect::<Vec<bool>>();

    let edge_is_quarantine = | index: EdgeIndex | {
        match edges.get(index).unwrap().data {
            EdgeData::Cir(_) => false,
            EdgeData::Sw(_) => true,
        }
    };

    nodes.iter().enumerate().for_each(|(node_index, _node)| {
        if visited_nodes[node_index] {
            return;
        }

        let group = plague_algo(node_index, &adjacent_node, &mut visited_nodes, edge_is_quarantine);
        basis.push(group);

    });

    let basis_eles = basis.iter().enumerate().map(|group| {
        Rc::new(BasisEle {
            id: group.0,
            nodes: group.1.iter().map(|node_index| nodes[*node_index].clone()).collect::<Vec<Rc<PsNode>>>()
        });
    });

    let to_b = iter::repeat(None).take(nodes.len()).collect::<Vec<Option<Rc<BasisEle>>>>();
    basis_eles.for_each(|bi| {
        bi.nodes.iter().for_each(|node| {
            to_b[node.index].replace(bi.clone());
        })
    });


    SigmAlg {
        to_basis: to_b.into_iter().map(|val| val.unwrap()).collect(),
        basis: basis_eles.collect()
    }
}



// pub fn generate_outage(ps: &PowerSystem, edge_names: Vec<String>) -> Outage {

//     let edge_indices = edge_names.iter().map(|en| ps.get_edge_by_name())
    

//     delta_u.iter().for_each(|du: &DeltaU| u[du.index] = du.new_u);

//     let mut visited_nodes = iter::repeat(false).take(ps.nodes.len()).collect::<Vec<bool>>();

//     let edge_is_quarantine = | index: EdgeIndex | {
//         ps.edges.get(index).unwrap().quarantines_super_node(&u.get(index))
//     };

//     ps.nodes_iter().enumerate().for_each(|(node_index, _node)| {
//         if visited_nodes[node_index] {
//             return;
//         }

//         let group = plague_algo(node_index, ps, &mut visited_nodes, edge_is_quarantine);
//         ret_val.push(group);

//     });

//     return ret_val;


//     return Outage { in_outage: vec![], boundary: vec![], delta_u: vec![] };

    
// }

pub fn generate_super_node_mapping(ps: &PowerSystem, delta_u: &Vec<DeltaU>) -> Vec<Vec<usize>> {
    let mut u = ps.start_u.clone();
    let mut ret_val = Vec::new();

    delta_u.iter().for_each(|du: &DeltaU| u[du.index] = du.new_u);

    let mut visited_nodes = iter::repeat(false).take(ps.nodes.len()).collect::<Vec<bool>>();

    let edge_is_quarantine = | index: EdgeIndex | {

        // println!("index {:?}", index);
        // println!("edge {:?}", ps.edges.get(index).unwrap());
        // println!("quarantines_super_node {:?}", ps.edges.get(index).unwrap().quarantines_super_node(&u.get(index)));
        ps.edges.get(index).unwrap().quarantines_super_node(&u.get(index))
    };

    ps.nodes_iter().enumerate().for_each(|(node_index, _node)| {
        if visited_nodes[node_index] {
            return;
        }

        let group = plague_algo(node_index, &ps.adjacent_node, &mut visited_nodes, edge_is_quarantine);
        ret_val.push(group);

    });

    return ret_val;

}


mod tests {
    use crate::power_system::EdgeData;

    use super::*;
    
    const BRB_FILE_PATH: &str = "./grids/BRB/"; 

    #[test]
    fn all_closed(){
        let ps = PowerSystem::from_files(BRB_FILE_PATH);

        let delta_u = ps.edges_iter()
        .enumerate()
        .filter(|e| match e.1.data {
            EdgeData::Cir(_) => false,
            EdgeData::Sw(_) => true,
        })
        .map(|e| { return DeltaU{ index: e.0, new_u: U::Closed }; })
        .collect::<Vec<DeltaU>>();

        let res = generate_super_node_mapping(&ps, &delta_u);

        println!("{:?}", res);

        assert_eq!(res.len(), 6);
    }

    #[test]
    fn all_open(){
        let ps = PowerSystem::from_files(BRB_FILE_PATH);

        let delta_u = ps.edges_iter()
        .enumerate()
        .filter(|e| match e.1.data {
            EdgeData::Cir(_) => false,
            EdgeData::Sw(_) => true,
        })
        .map(|e| { return DeltaU{ index: e.0, new_u: U::Open }; })
        .collect::<Vec<DeltaU>>();

        let res = generate_super_node_mapping(&ps, &delta_u);

        println!("{:?}", res);

        assert_eq!(res.len(), ps.nodes.len());
    }

    #[test]
    fn sigma_alg(){
        let ps = PowerSystem::from_files(BRB_FILE_PATH);

        let sig: SigmAlg = generate_sigma_alg(&ps.adjacent_node, &ps.edges, &ps.nodes);

        println!("{:?}", sig);    
        
    }

}
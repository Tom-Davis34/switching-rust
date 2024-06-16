// use std::{
//     collections::HashSet, iter::{self, zip}, rc::{Rc, Weak}
// };

// use nalgebra::ComplexField;
// use num_traits::Zero;

// use crate::traits::C32;

// use super::{
//     plague_algo::{plague_algo, SigAlg, SimpleSigAlg, create_sigma_alg},
//     DeltaU, Edge, EdgeData, EdgeIndex, NodeType, PowerSystem, PsNode, U, EdgeData::Sw, EdgeData::Cir
// };

// #[derive(Debug, PartialEq, Clone)]
// pub struct VectorIndex(usize);

// #[derive(Debug, Clone)]
// pub struct PfNode {
//     pub index: VectorIndex,
//     pub pq: C32,
//     pub n_type: NodeType,
//     pub is_load: bool,
//     pub system_v: f32,
//     pub unit_v: C32,
//     pub is_dead: bool,
//     pub nodes: Vec<Rc<PsNode>>,
//     pub edges: Vec<Weak<PfEdge>>,
// }

// impl PfNode {
//     pub fn pq_is_zero(&self) -> bool{
//         return self.pq.is_zero()
//     }
// }

// impl PartialEq for PfNode {
//     fn eq(&self, other: &Self) -> bool {
//         self.index == other.index
//     }
// }

// #[derive(Debug, Clone)]
// pub struct PfEdge {
//     pub index: VectorIndex,
//     pub edge: Rc<Edge>,
//     pub tnode: Weak<PfNode>,
//     pub fnode: Weak<PfNode>,
//     pub unit_current: C32,
//     pub admittance: C32,
// }

// impl PartialEq for PfEdge {
//     fn eq(&self, other: &Self) -> bool {
//         self.index == other.index
//     }
// }

// #[derive(Debug, Clone)]
// pub struct PfGraph<'a> {
//     pub nodes: Vec<Rc<PfNode>>,
//     pub ps_node_to_pf_node: Vec<Option<Rc<PfNode>>>,
//     pub edges: Vec<Rc<PfEdge>>,
//     pub ps_edge_to_pf_edge: Vec<Option<Rc<PfEdge>>>,
//     pub ps: &'a PowerSystem
// }

// impl PfGraph {
//     pub fn live_nodes(&self) -> impl Iterator<Item = &Rc<PfNode>> {
//         self.nodes.iter().filter(|n| !n.is_dead)
//     }
// }

// pub fn prune_dead_nodes(pf_graph: PfGraph) -> PfGraph {
//     let mut nodes_temp = pf_graph.live_nodes().filter(|n| n.n_type == NodeType::Sk).collect::<Vec<&Rc<PfNode>>>();
//     assert!(nodes_temp.len() == 1);

//     pf_graph.live_nodes().filter(|n| n.n_type == NodeType::PV).for_each(|n| {
//         nodes_temp.push(n);
//     });

//     pf_graph.live_nodes().filter(|n| n.n_type == NodeType::PQ).for_each(|n| {
//         nodes_temp.push(n);
//     });

//     let edges = pf_graph.edges.iter().map(|e| {
//         Rc:new(PfEdge {
//             index: todo!(),
//             edge: todo!(),
//             tnode: todo!(),
//             fnode: todo!(),
//             unit_current: todo!(),
//             admittance: todo!(),
//         })
//     }).collect();

//     let nodes2 = nodes_temp.iter().enumerate().map(|(index, n)| 
//         Rc::new(PfNode {
//             index: VectorIndex(index),
//             pq: n.pq,
//             n_type: n.n_type,
//             is_load: n.is_load,
//             system_v: n.system_v,
//             unit_v: n.unit_v,
//             is_dead: n.is_dead,
//             nodes: n.nodes.clone(),
//             edges: n.edges.iter().filter(|e| !e.upgrade().unwrap().fnode.upgrade().unwrap().is_dead && !e.upgrade().unwrap().tnode.upgrade().unwrap().is_dead)
//                 .map(|e| e.clone()).collect(),
//         })
//     ).collect::<Vec<Rc<PfNode>>>();

//     return pf_graph;
// }

// pub fn generate_pf_graph<'a>(ps: &'a PowerSystem, u: &'a Vec<U>) -> PfGraph<'a> {
//     let edge_u = ps.edges
//     .iter()
//     .map(|ed| (ed.clone(), u.get(ed.index).map(|u| u.clone())))
//         .collect::<Vec<(Rc<Edge>, Option<U>)>>();

//     let edge_is_quarantine = |index: EdgeIndex| {
//         let eu = edge_u.get(index).unwrap();

//         match eu.0.data {
//             EdgeData::Cir(_) => true,
//             EdgeData::Sw(_) => eu.1.unwrap() == U::Open,
//         }
//     };

//     let sig_alg = create_sigma_alg(&ps, &edge_is_quarantine);

//     let live_nodes = find_live_nodes(ps, u);

//     let pf_nodes_temp = generate_pf_nodes_temp(&sig_alg, &live_nodes);

//     let pf_edges = create_pf_edges(&ps, &sig_alg, &pf_nodes_temp);

//     // println!("edges {:#?}", pf_edges);

//     let pf_nodes = create_pf_nodes(pf_nodes_temp, &pf_edges);

//     let ps_edge_to_pf_edge = create_ps_edge_to_pf_edge(ps, &pf_edges);

//     let ps_node_to_pf_node = create_ps_node_to_pf_node(ps, &pf_nodes, sig_alg);

//     return PfGraph {
//         nodes: pf_nodes,
//         ps_node_to_pf_node,
//         edges: pf_edges,
//         ps_edge_to_pf_edge,
//         ps,
//     };
// }

// fn create_pf_nodes(pf_nodes_temp: Vec<Rc<PfNode>>, pf_edges: &Vec<Rc<PfEdge>>) -> Vec<Rc<PfNode>> {
//     let pf_nodes: Vec<Rc<PfNode>> = pf_nodes_temp
//         .iter()
//         .map(|pf_node| {
//             let edges = pf_edges
//                 .iter()
//                 .filter(|e| {
//                     // println!("e.tnode.upgrade() {:#?}", e.tnode.upgrade());
//                     // println!("e.fnode.upgrade() {:#?}", e.fnode.upgrade());

//                     e.tnode.upgrade().unwrap() == *pf_node || e.fnode.upgrade().unwrap() == *pf_node
//                 })
//                 .map(Rc::downgrade)
//                 .collect::<Vec<Weak<PfEdge>>>();

//             Rc::new(PfNode {
//                 index: pf_node.index.clone(),
//                 pq: pf_node.pq.clone(),
//                 is_load: pf_node.is_load,
//                 system_v: pf_node.system_v,
//                 unit_v: pf_node.unit_v,
//                 nodes: pf_node.nodes.iter().map(|n| n.clone()).collect::<Vec<Rc<PsNode>>>(),
//                 edges: edges,
//                 n_type: pf_node.nodes.iter().map(|n| n.n_type.clone()).max().unwrap(),
//                 is_dead: pf_node.is_dead
//             })
//         })
//         .collect();
//     pf_nodes
// }

// fn create_ps_edge_to_pf_edge(ps: &PowerSystem, pf_edges: &Vec<Rc<PfEdge>>) -> Vec<Option<Rc<PfEdge>>> {
//     let mut ps_edge_to_pf_edge = ps
//         .edges
//         .iter()
//         .map(|_n| None)
//         .collect::<Vec<Option<Rc<PfEdge>>>>();

//     for ele in pf_edges.iter() {
//         ps_edge_to_pf_edge[ele.edge.index] = Some(ele.clone());
//     }
//     ps_edge_to_pf_edge
// }

// fn create_ps_node_to_pf_node(ps: &PowerSystem, pf_nodes: &Vec<Rc<PfNode>>, sig_alg: SimpleSigAlg) -> Vec<Option<Rc<PfNode>>> {
//     let ps_node_to_pf_node = ps
//         .nodes
//         .iter()
//         .enumerate()
//         .map(|(_i, node)| Some(
//          pf_nodes[sig_alg.get_basis(node).index].clone()
//     ))
//         .collect::<Vec<Option<Rc<PfNode>>>>();
//     ps_node_to_pf_node
// }

// fn find_live_nodes(ps: &PowerSystem, u: &Vec<U>) -> HashSet<usize> {
//     let mut infected = ps.nodes.iter().map(|_n| false).collect();
//     let slack_node = ps.nodes_iter().find(|n| n.n_type == NodeType::Sk).map(|n| n.index).unwrap(); 

//     let edge_is_quarantine = |index: EdgeIndex| {
//         !(index >= u.len() || u[index] == U::Closed)
//     };

//     let live_nodes = plague_algo(slack_node, &ps.adjacent_node, &mut infected, edge_is_quarantine);

//     return live_nodes.iter().map(usize::clone).collect();
// }

// fn create_pf_edges(
//     ps: &PowerSystem,
//     sig_alg: &SimpleSigAlg,
//     pf_nodes: &Vec<Rc<PfNode>>
// ) -> Vec<Rc<PfEdge>> {
//     let ps_node_to_pf_node = ps
//         .nodes
//         .iter()
//         .enumerate()
//         .map(|(_i, node)| pf_nodes[sig_alg.get_basis(node).index].clone())
//         .collect::<Vec<Rc<PfNode>>>();

//     let edges: Vec<Rc<PfEdge>> = ps
//         .edges
//         .iter()
//         .filter(|edge| {
//             ps_node_to_pf_node[edge.tbus.index].index != ps_node_to_pf_node[edge.fbus.index].index
//         })
//         .enumerate()
//         .map(|(index, edge)| {
//             let pf_tnode = &ps_node_to_pf_node[edge.tbus.index];
//             let pf_fnode = &ps_node_to_pf_node[edge.fbus.index];

//             Rc::new(PfEdge {
//                 index: VectorIndex(index),
//                 edge: edge.clone(),
//                 tnode: Rc::downgrade(&pf_tnode),
//                 fnode: Rc::downgrade(&pf_fnode),
//                 unit_current: C32::new(0.0, 0.0),
//                 admittance: edge.admittance(),
//             })
//         })
//         .collect();

//     return edges;
// }

// fn generate_pf_nodes_temp(
//     sig_alg: &SimpleSigAlg,
//     live_nodes: &HashSet<usize>) -> Vec<Rc<PfNode>> {
//     sig_alg
//         .basis
//         .iter()
//         .enumerate()
//         .map(|(index, basis)| {
//             let pq = basis.nodes.iter().map(|n| n.gen - n.load).sum();
//             let is_load = basis.nodes.iter().any(|n| n.load != C32::new(0.0, 0.0));
//             let system_v = basis.nodes.first().unwrap().system_v;

//             Rc::new(PfNode {
//                 index: VectorIndex(index),
//                 pq,
//                 n_type: basis.nodes.iter().map(|n| n.n_type.clone()).max().unwrap(),
//                 is_load,
//                 system_v,
//                 unit_v: C32::new(0.0, 0.0),
//                 nodes: basis
//                     .nodes
//                     .iter()
//                     .map(|n| n.clone())
//                     .collect::<Vec<Rc<PsNode>>>(),
//                 edges: vec![],
//                 is_dead: !live_nodes.contains(&index)
//             })
//         })
//         .collect::<Vec<Rc<PfNode>>>()
// }



// mod tests {
//     use crate::power_system::PowerSystem;

//     use super::*;

//     const BRB_FILE_PATH: &str = "./grids/BRB/";

//     #[test]
//     fn create_pf_graph_all_open() {
//         let ps = PowerSystem::from_files(BRB_FILE_PATH);

//         let u = ps.switch_iter().map(|sw| match sw.data{
//             Sw(_) => U::Open,
//             Cir(_) => panic!(),
//         })
//         .collect();
        
//         let pf_graph = generate_pf_graph(&ps, &u);
        
//         let dead_nodes = pf_graph.nodes.iter().filter(|n| n.is_dead).collect::<Vec<&Rc<PfNode>>>();
//         let live_nodes = pf_graph.nodes.iter().filter(|n| !n.is_dead).collect::<Vec<&Rc<PfNode>>>();

//         assert_eq!(dead_nodes.len(), 27);
//         assert_eq!(live_nodes.len(), 3);

//         // println!("{:#?}", pf_graph.nodes);
//     }


//     #[test]
//     fn create_pf_graph_all_closed() {
//         let ps = PowerSystem::from_files(BRB_FILE_PATH);

//         let u = ps.switch_iter().map(|sw| match sw.data{
//             Sw(_) => U::Closed,
//             Cir(_) => panic!(),
//         })
//         .collect();
        
//         let pf_graph = generate_pf_graph(&ps, &u);

//         // println!("{:#?}", pf_graph.nodes.iter().map(|n| n.index.clone()).collect::<Vec<VectorIndex>>());
//         // println!("{:#?}", pf_graph.ps_node_to_pf_node.iter().map(|n| n.index.clone()).collect::<Vec<VectorIndex>>());
        
//         let dead_nodes = pf_graph.nodes.iter().filter(|n| n.is_dead).collect::<Vec<&Rc<PfNode>>>();
//         let live_nodes = pf_graph.nodes.iter().filter(|n| !n.is_dead).collect::<Vec<&Rc<PfNode>>>();

//         assert_eq!(dead_nodes.len(), 0);
//         assert_eq!(live_nodes.len(), 6);

//         assert_eq!(pf_graph.nodes[0].nodes.len(), 25);
//         assert_eq!(pf_graph.nodes[0].pq, C32::new(-50.0, -10.0));
//         let node = &pf_graph.ps_node_to_pf_node[25];
//         assert_eq!(node.pq, C32::new(-250.0, -80.0));

//         assert_eq!(&pf_graph.ps_node_to_pf_node[26].unwrap().n_type, &NodeType::Sk);
//         // println!("{:#?}", pf_graph.nodes);
//     }

//     #[test]
//     fn check_impedance_clacs_pf_graph_all_closed() {
//         let ps = PowerSystem::from_files(BRB_FILE_PATH);

//         let u = ps.switch_iter().map(|sw| match sw.data{
//             Sw(_) => U::Closed,
//             Cir(_) => panic!(),
//         })
//         .collect();
        
//         let pf_graph = generate_pf_graph(&ps, &u);

//         // println!("{:#?}", pf_graph.nodes.iter().map(|n| n.index.clone()).collect::<Vec<VectorIndex>>());
//         // println!("{:#?}", pf_graph.ps_node_to_pf_node.iter().map(|n| n.index.clone()).collect::<Vec<VectorIndex>>());
        
//         let dead_nodes = pf_graph.nodes.iter().filter(|n| n.is_dead).collect::<Vec<&Rc<PfNode>>>();
//         let live_nodes = pf_graph.nodes.iter().filter(|n| !n.is_dead).collect::<Vec<&Rc<PfNode>>>();

//         assert_eq!(dead_nodes.len(), 0);
//         assert_eq!(live_nodes.len(), 6);

//         assert_eq!(pf_graph.nodes[0].nodes.len(), 25);
//         assert_eq!(pf_graph.nodes[0].pq, C32::new(-50.0, -10.0));
//         let node = &pf_graph.ps_node_to_pf_node[25];
//         assert_eq!(node.pq, C32::new(-250.0, -80.0));

//         assert_eq!(&pf_graph.ps_node_to_pf_node[26].unwrap().n_type, &NodeType::Sk);





//         // println!("{:#?}", pf_graph.nodes);
//     }
// }
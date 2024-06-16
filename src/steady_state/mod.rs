use chrono::Duration;
use nalgebra::DVector;

use crate::{graph::{NodeIndex, transform::{CreateSubGraph, SubGraphMap}, Graph}, power_system::{PowerSystem, PsNode, PsEdge, U}, traits::C32};

use self::solve::steady_state_solve;

mod solve;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SteadyStateError {
    Msg(String),
    NonConvergence,
    Divergence
}

#[derive(Clone, Debug)]
pub struct SteadyStateStats {
    iter_count: u32,
}

#[derive(Debug)]
pub struct SteadyStateResults {
    pub graph: Graph<PsNode, PsEdge>, 
    pub sub_graph_map: SubGraphMap,
    pub sub_v: DVector<C32>,
    pub super_v: DVector<Option<C32>>,
    pub stats: SteadyStateStats,
}

fn create_sub_graph(ps: &PowerSystem, u_vec: &Vec<U>) -> (Graph<PsNode, PsEdge>, SubGraphMap){
    
    let live_nodes = ps.live_nodes(u_vec);
    println!("live nodes {:#?}",live_nodes);
    let nm = |n: &PsNode| n.clone();
    let nf = |n: &PsNode| live_nodes.contains(&n.index);
    let em = |e: &PsEdge| e.clone();
    let mut subgraph_creator = CreateSubGraph::new(&ps.g, nm, nf, em);

    let edge_contraction_node_merge = |_e: &PsEdge, fnode: &PsNode, tnode: &PsNode | {
        PsNode {
            num: tnode.num,
            index: tnode.index,
            load: fnode.load + tnode.load,
            gen: fnode.gen + tnode.gen,
            system_v: tnode.system_v,
            n_type: fnode.n_type.max(tnode.n_type),
        }
    };

    let edge_contraction_edge_filter = |e: &PsEdge | { (e.is_switch() && e.u == U::Closed) ||  e.admittance().re == 0.0 && e.admittance().im == 0.0 };

    subgraph_creator.edge_contraction_filter(&edge_contraction_node_merge, &edge_contraction_edge_filter);
    return subgraph_creator.complete();
}

pub fn steady_state_pf(ps: &PowerSystem, u_vec: &Vec<U>) -> Result<SteadyStateResults, SteadyStateError> {

    let (simplier_graph, sub) =  create_sub_graph(ps, u_vec);
    let sub_v = steady_state_solve(&simplier_graph)?;

    let super_v = map_to_super_v(&sub, &sub_v.v, ps.node_count());

    Ok(SteadyStateResults{
        graph: simplier_graph,
        sub_graph_map: sub,
        sub_v: sub_v.v,
        super_v: super_v,
        stats: SteadyStateStats { iter_count: sub_v.iter_count},
    })

}

fn map_to_super_v(sub: &SubGraphMap, sub_v: &DVector<C32>, super_size: usize) -> DVector<Option<C32>> {
    DVector::<Option<C32>>::from_fn(super_size, |r, _c| { 
        sub.get_sub_node(NodeIndex(r)).map(|index| sub_v.get(index.0).unwrap().clone())
    })
}

mod tests {
    use std::{clone, convert::identity};

    use nalgebra_sparse::SparseEntry::Zero;

    use super::*;
    use crate::power_system::{*, self};

    const SIMPLE_STEADY_STATE_FILE_PATH: &str = "./grids/SimpleSteadyState/";
    const SIMPLE_STEADY_STATE_2_FILE_PATH: &str = "./grids/SimpleSteadyState2/";

    #[test]
    fn create_sub_graph_test(){
        let ps = PowerSystem::from_files(SIMPLE_STEADY_STATE_2_FILE_PATH);

        let u_vec = vec![U::DontCare, U::DontCare, U::DontCare];
        let (sub_graph, map) = super::create_sub_graph(&ps, &u_vec);

        println!("sub_graph {:#?}", sub_graph);
        println!("map {:#?}", map);

    }

    #[test]
    fn steady_state_pf_test(){
        let ps = PowerSystem::from_files(SIMPLE_STEADY_STATE_2_FILE_PATH);
        
        let u_vec = vec![U::DontCare, U::DontCare, U::DontCare];
        let res = super::steady_state_pf(&ps, &u_vec);

        // println!("res {:#?}", res);
        let ss_res = res.unwrap();

        let vec = ss_res.sub_v;
        assert_eq!(vec.get(0).unwrap(), &C32::new(1.0,0.0));
        assert_eq!(vec.get(1).unwrap(), &C32::new(1.0156636,0.025887817));
        assert_eq!(vec.get(2).unwrap(), &C32::new(0.97542495,-0.020732194));

        let vec_super = ss_res.super_v;
        assert_eq!(vec_super.get(0).unwrap().unwrap(), C32::new(1.0,0.0));
        assert_eq!(vec_super.get(1).unwrap().unwrap(), C32::new(1.0156636,0.025887817));
        assert_eq!(vec_super.get(2).unwrap().unwrap(), C32::new(0.97542495,-0.020732194));
    }
}
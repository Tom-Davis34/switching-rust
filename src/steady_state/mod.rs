use std::rc::Rc;

use crate::{power_system::{PowerSystem, power_flow_model::{PfGraph, PfNode}, NodeType}, traits::C32};


struct SsNode {
    index: usize,
    cur: C32,
    cap: f32,
    y_g: C32,
    v: C32,    
    pf_node: Rc<PfNode>,
    n_type:NodeType,
}

pub fn steady_state_pf(mut pf_graph: PfGraph) -> PfGraph {
    
    let mut v1_nodes: Vec<SsNode> = Vec::new();
    pf_graph.live_nodes().filter(|n| n.n_type == NodeType::Sk).for_each(|n|{
        v1_nodes.push(SsNode { index: v1_nodes.len(), cur: C32::new(0.0, 0.0), v: C32::new(1.0, 0.0), pf_node: n.clone(), n_type: NodeType::Sk, cap: todo!(), y_g: todo!() });
    });
    let mut v1_nodes: Vec<SsNode> = Vec::new();
    pf_graph.live_nodes().filter(|n| n.n_type == NodeType::PV).for_each(|n|{
        v1_nodes.push(SsNode { index: v1_nodes.len(), cur: C32::new(0.0, 0.0), v: C32::new(0.9, 0.0), pf_node: n.clone(), n_type: NodeType::PV })
    });
    pf_graph.live_nodes().filter(|n| n.n_type == NodeType::PQ).for_each(|n|{
        v1_nodes.push(SsNode { index: v1_nodes.len(), cur: C32::new(0.0, 0.0), v: C32::new(0.9, 0.0), pf_node: n.clone(), n_type: NodeType::PQ })
    });
    
    

    return pf_graph;
}
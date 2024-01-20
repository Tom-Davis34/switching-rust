use std::collections::BinaryHeap;

use crate::power_system::{DeltaU, PowerSystem, self, U};

use self::a_star_node::AStarNode;

pub mod a_star_node;
mod steady_state_adapter;
mod transient_adapter;

const HAMMING_DIST_SCALE: f32 = 10.0;

pub fn a_star(ps: &PowerSystem, target_du: Ve<U>) -> AStarNode {

    let mut heap = BinaryHeap::new();

    let start_h = HAMMING_DIST_SCALE * U::hamming_dist(&target_du, &ps.start_u);
    let root = AStarNode::new(None, None, start_h);
    heap.push(root);

    loop {
        let mut current_node = heap.pop().unwrap();

        println!("current_node {:#?}", current_node);

        if current_node.h == 0.0 {
            return current_node;
        }

        match current_node.state{
            a_star_node::NodeState::Init => {
                let u = create_u_from_node(ps, &current_node);
                let res = steady_state_adapter::compute_ss_contri(ps, &u);
                current_node.add_steady_state(res.0, res.1);

                heap.push(current_node);
            },
            a_star_node::NodeState::SteadyStateCalculated => {
                let u = create_u_from_node(ps, &current_node);
                let res = transient_adapter::compute_trans_contri(ps, &u);
                current_node.add_transient(res.0, res.1);

                heap.push(current_node);
            },
            a_star_node::NodeState::TransientCalculated => {
                let u = create_u_from_node(ps, &current_node);

                u.iter().enumerate().for_each(|(index, u)| {
                    
                    AStarNode::new(None, None, start_h);
                })
            },
            a_star_node::NodeState::Finished =>{
                panic!("Finished node added to the heap");
            },
        }
    }
}

fn create_u_from_node(ps: &PowerSystem, node: &AStarNode) -> Vec<U> {
    let mut u = ps.start_u.clone();
    node.get_delta_u().iter().for_each(|du: &DeltaU| u[du.index] = du.new_u);
    return u;
}

fn create_u_from_parent(ps: &PowerSystem, parent: &AStarNode, new_delta_u: DeltaU) -> Vec<U> {
    let mut u = create_u_from_node(ps, parent);
     u[new_delta_u.index] = new_delta_u.new_u;
    
    return u;
}

fn compute_h_from_parent(ps: &PowerSystem, parent: &AStarNode, new_delta_u: DeltaU, target_u: Vec<U>) -> f32{
    let actual_u = create_u_from_parent(ps, parent, new_delta_u);    

    return U::hamming_dist(&target_u, &actual_u);
}

fn compute_h(ps: &PowerSystem, node: &AStarNode, target_u: Vec<U>) -> f32{
    let actual_u = create_u_from_node(ps, &node);    

    return U::hamming_dist(&target_u, &actual_u);
}

// fn create_a_star_node(ps: &PowerSystem, parent: &AStarNode, new_delta_u: DeltaU){


//     AStarNode::new(None, None, U::hamming_dist(target_u, actual_u));
   
// }


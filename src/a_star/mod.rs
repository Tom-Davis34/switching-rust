use std::{collections::BinaryHeap, rc::Rc, cell::RefCell};

use nalgebra::uninit::Init;

use crate::power_system::{DeltaU, PowerSystem, self, U};

use self::a_star_node::{AStarNode, HeapNode};

pub mod a_star_node;
mod steady_state_adapter;
mod transient_adapter;

const HAMMING_DIST_SCALE: f32 = 10.0;



pub fn a_star(ps: &PowerSystem, target_du: Vec<U>) -> HeapNode {

    let mut heap = BinaryHeap::new();

    let start_h = HAMMING_DIST_SCALE * U::hamming_dist(&target_du, &ps.start_u);
    let root: HeapNode = Rc::new(RefCell::new(AStarNode::new(None, None, start_h)));
    heap.push(root);

    loop {
        let current_node = heap.pop().unwrap();

        println!("current_node {:#?}", current_node);

        if current_node.borrow().h == 0.0 {
            return current_node;
        }

        handle_node(current_node, ps, &target_du, &mut heap);
    }
}

fn handle_node(current_node: HeapNode, ps: &PowerSystem, target_du: &Vec<U>, heap: &mut BinaryHeap<HeapNode>) {
    match current_node.borrow().state{
        a_star_node::NodeState::Init => {
            let u = create_u_from_node(ps, &current_node);
            let res = steady_state_adapter::compute_ss_contri(ps, &u);
            current_node.borrow_mut().add_steady_state(res.0, res.1);
        
            heap.push(current_node.clone());
        },
        a_star_node::NodeState::SteadyStateCalculated => {
            let u = create_u_from_node(ps, &current_node);
            let res = transient_adapter::compute_transient_contri(ps, &u);
            current_node.borrow_mut().add_transient(res.0, res.1);
        
            heap.push(current_node.clone());
        },
        a_star_node::NodeState::TransientCalculated => {
            let mut actual_u: Vec<U> = create_u_from_node(ps, &current_node);

            for index in 0..actual_u.len() {
                let u = actual_u[index];

                let not_u = DeltaU {
                    index,
                    new_u: u.not(),
                };

                actual_u[index] = u.not(); 
                let new_node = AStarNode::new(
                    Some(current_node.clone()),
                    Some(not_u.clone()),
                    HAMMING_DIST_SCALE * U::hamming_dist(&target_du, &actual_u),
                );
                actual_u[index] = u; 

                heap.push(Rc::new(RefCell::new(new_node)));
            }
        },
        a_star_node::NodeState::Finished => panic!("Why is this state 'Finished' reached?"),
    }
}

fn create_u_from_node(ps: &PowerSystem, node: &HeapNode) -> Vec<U> {
    let mut u = ps.start_u.clone();
    AStarNode::get_delta_u(node).iter().for_each(|du: &DeltaU| u[du.index] = du.new_u);
    return u;
}

fn create_u_from_parent(ps: &PowerSystem, parent: &HeapNode, new_delta_u: DeltaU) -> Vec<U> {
    let mut u = create_u_from_node(ps, parent);
     u[new_delta_u.index] = new_delta_u.new_u;
    
    return u;
}

fn compute_h_from_parent(ps: &PowerSystem, parent: &HeapNode, new_delta_u: DeltaU, target_u: Vec<U>) -> f32{
    let actual_u = create_u_from_parent(ps, parent, new_delta_u);    

    return U::hamming_dist(&target_u, &actual_u);
}

fn compute_h(ps: &PowerSystem, node: &HeapNode, target_u: Vec<U>) -> f32{
    let actual_u = create_u_from_node(ps, &node);    

    return U::hamming_dist(&target_u, &actual_u);
}

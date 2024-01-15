use std::collections::BinaryHeap;

use crate::power_system::{DeltaU, PowerSystem, self, U};

use self::a_star_node::AStarNode;

pub mod a_star_node;
mod steady_state_adapter;

const HAMMING_DIST_SCALE: f32 = 10.0;

pub fn a_star(ps: &PowerSystem, target_du: Vec<U>) -> AStarNode {

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
            },
            a_star_node::NodeState::SteadyStateCalculated => todo!(),
            a_star_node::NodeState::TransientCalculated => todo!(),
            a_star_node::NodeState::Finished => todo!(),
        }
    }
    // dist[start] = 0;
    // heap.push(State { cost: 0, position: start });

    // // Examine the frontier with lower cost nodes first (min-heap)
    // while let Some(State { cost, position }) = heap.pop() {
    //     // Alternatively we could have continued to find all shortest paths
    //     if position == goal { return Some(cost); }

    //     // Important as we may have already found a better way
    //     if cost > dist[position] { continue; }

    //     // For each node we can reach, see if we can find a way with
    //     // a lower cost going through this node
    //     for edge in &adj_list[position] {
    //         let next = State { cost: cost + edge.cost, position: edge.node };

    //         // If so, add it to the frontier and continue
    //         if next.cost < dist[next.position] {
    //             heap.push(next);
    //             // Relaxation, we have now found a better way
    //             dist[next.position] = next.cost;
    //         }
    //     }
    // }

    // // Goal not reachable
    // None

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


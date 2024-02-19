use std::{cell::RefCell, collections::BinaryHeap, fmt::{Binary, Debug, Display}, rc::Rc, ops::Add};

use chrono::{DateTime, Utc, Duration};
use nalgebra::uninit::Init;

use crate::{power_system::{self, DeltaU, PowerSystem, U, Outage}, a_star::a_star_node::NodeState, utils::{duration, PrettyDuration}};

use self::{a_star_node::{AStarNode, HeapNode, Contribution}, steady_state_adapter::SteadyStateContri, transient_adapter::TransientAdapter};

pub mod a_star_node;
mod steady_state_adapter;
mod transient_adapter;

const HAMMING_DIST_SCALE: f32 = 10.0;

// pub struct Os {
//     pub osis: Vec<Osi>,
// }

// impl Debug for Osi {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.display)
//     }
// }

#[derive(Debug, PartialEq, Clone)]
pub struct OS(Vec<HeapNode>);

impl Display for OS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for osi in &self.0[0..self.0.len() - 1] {
            writeln!(f, "{}", osi.borrow().display)?
        }

        Ok(())
    }
}

pub struct LogHeapNode(HeapNode);

impl Display for LogHeapNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let node = self.0.borrow();

        writeln!(f, "Display:            {}", node.display)?;
        writeln!(f, "State:              {:?}", node.state)?;
        writeln!(f, "H:                  {:?}", node.h)?;
        writeln!(f, "Objective:          {:?}", node.objective)?;
        match &node.steady_state_contri {
            Some(ssc) => writeln!(f, "Steady State Contri:{:?}", ssc.contri.iter().map(|c| c.amount).sum::<f32>())?,
            None => {},
        }
        match &node.transient_contri {                  
            Some(tc) => writeln!(f, "Transient Contri:   {:?}", tc.contri.iter().map(|c| c.amount).sum::<f32>())?,
            None => {},
        }            
        writeln!(f, "Depth:              {:?}", node.depth)
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct OSI(HeapNode);

impl Display for OSI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.0.borrow().display)
    }
}



#[derive(Debug, PartialEq, Clone)]
pub struct AStarStats {
    pub total_nodes: u32,
    pub ss_num: u32,
    pub ss_duration: Duration,
    pub transient_num: u32,
    pub transient_duration: Duration,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

impl Display for AStarStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, " === AStarStats === ")?;
        writeln!(f, "total nodes:        {}", self.total_nodes)?;
        writeln!(f, "ss calcs:           {}", self.ss_num)?;
        writeln!(f, "ss time:            {}", PrettyDuration(self.ss_duration))?;
        writeln!(f, "transient calcs:    {}", self.transient_num)?;
        writeln!(f, "transient time:     {}", PrettyDuration(self.transient_duration))?;
        let dur = duration(&self.start_time, &self.end_time);
        match dur {
            Some(d) => writeln!(f, "total time:         {}", PrettyDuration(d)),
            None => Ok(()),
        }
    }
}


#[derive(Debug)]
pub struct AStar {
    pub stats: AStarStats,
    pub heap: BinaryHeap<HeapNode>,
    pub os: Option<OS>,
}

impl AStar {
    pub fn new() -> AStar {
        return AStar {
            stats: AStarStats {
                total_nodes: 0,
                ss_num: 0,
                ss_duration: Duration::milliseconds(0),
                transient_num: 0,
                transient_duration: Duration::milliseconds(0),
                start_time: None,
                end_time: None,
            },
            heap: BinaryHeap::new(),
            os: None,
        };
    }

    pub fn run(mut self, ps: &PowerSystem, outage: &Outage) -> Self {
        self.stats.start_time = Some(Utc::now());

        let start_h = HAMMING_DIST_SCALE * U::hamming_dist(&outage.target_u, &ps.start_u);
        let root: HeapNode = Rc::new(RefCell::new(AStarNode::new(None, None, start_h, ps)));
        self.heap.push(root);

        let best_fit = self.main_loop(ps, &outage.target_u);

        let os_heap_nodes = AStarNode::get_nodes(&best_fit).iter().filter(|n| n.borrow().delta_u.is_some()).map(|n| n.clone()).collect::<Vec<HeapNode>>();
        
        self.os = Some(OS(os_heap_nodes));

        self.stats.end_time = Some(Utc::now());

        self
    }

    fn main_loop(&mut self, ps: &PowerSystem, target_du: &Vec<U>) -> HeapNode {

        loop {
            let current_node = self.heap.pop().unwrap();

            println!("{}", LogHeapNode(current_node.clone()));

            if current_node.borrow().h == 0.0 && current_node.borrow().state == NodeState::TransientCalculated {
                return current_node;
            }

            self.handle_node(current_node, ps, &target_du);
        }
    }

    fn handle_node(
        &mut self,
        current_node: HeapNode,
        ps: &PowerSystem,
        target_du: &Vec<U>
    ) {
        let state = current_node.borrow().state.clone();

        match state {
            a_star_node::NodeState::Init => {
                self.stats.ss_num += 1;

                let u = create_u_from_node(ps, &current_node);
                let contri = steady_state_adapter::compute_ss_contri(ps, &u);
                self.stats.ss_duration = self.stats.ss_duration.add(contri.duration);
                current_node.borrow_mut().add_steady_state(contri);
                
                self.heap.push(current_node.clone());
            }
            a_star_node::NodeState::SteadyStateCalculated => {
                self.stats.transient_num += 1;

                let u = create_u_from_node(ps, &current_node);
                let res = transient_adapter::compute_transient_contri(ps, &u);
                self.stats.transient_duration = self.stats.transient_duration.add(res.duration);
                current_node.borrow_mut().add_transient(res);

                self.heap.push(current_node.clone());
            }
            a_star_node::NodeState::TransientCalculated => {
                let mut actual_u: Vec<U> = create_u_from_node(ps, &current_node);

                self.stats.total_nodes += (actual_u.len() - 1) as u32;

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
                        ps,
                    );
                    actual_u[index] = u;

                    self.heap.push(Rc::new(RefCell::new(new_node)));
                }
            }
            a_star_node::NodeState::Finished => panic!("Why is this state 'Finished' reached?"),
        }
    }
}

impl Display for AStar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Stats {}", self.stats)?;
        match self.os {
            Some(ref os) => {
                writeln!(f, "OS: {}", os)
            }
            None => Ok(()),
        }
    }
}

fn create_u_from_node(ps: &PowerSystem, node: &HeapNode) -> Vec<U> {
    let mut u = ps.start_u.clone();
    AStarNode::get_delta_u(node)
        .iter()
        .for_each(|du: &DeltaU| u[du.index] = du.new_u);
    return u;
}

fn create_u_from_parent(ps: &PowerSystem, parent: &HeapNode, new_delta_u: DeltaU) -> Vec<U> {
    let mut u = create_u_from_node(ps, parent);
    u[new_delta_u.index] = new_delta_u.new_u;

    return u;
}

fn compute_h_from_parent(
    ps: &PowerSystem,
    parent: &HeapNode,
    new_delta_u: DeltaU,
    target_u: Vec<U>,
) -> f32 {
    let actual_u = create_u_from_parent(ps, parent, new_delta_u);

    return U::hamming_dist(&target_u, &actual_u);
}

fn compute_h(ps: &PowerSystem, node: &HeapNode, target_u: Vec<U>) -> f32 {
    let actual_u = create_u_from_node(ps, &node);

    return U::hamming_dist(&target_u, &actual_u);
}

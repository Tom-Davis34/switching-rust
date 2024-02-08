use std::{rc::Rc, cmp::Ordering, ops::{DerefMut, Deref}, cell::RefCell, borrow::BorrowMut};

use crate::power_system::{DeltaU, SigmAlg, plague_algo::PowerFlowNode, U};

use super::{steady_state_adapter::{SteadyStateResults, SteadyStateError}, transient_adapter::{TransientError, TransientResults}};

pub type HeapNode = Rc<RefCell<AStarNode>>;

#[derive(Debug, PartialEq,Clone)]
pub enum ContributionType{
    Other,
    SteadyState,
    Transient
}

#[derive(Debug, PartialEq, Clone)]
pub struct Contribution {
    pub contri_type: ContributionType,
    pub reason: String,
    pub amount: f32,
}

#[derive(Debug, PartialEq, Clone)]
pub enum NodeState{
	Init,
	SteadyStateCalculated,
	TransientCalculated,
	Finished 
}

#[derive(Debug, PartialEq, Clone)]
pub struct AStarNode {
    pub state: NodeState,
    pub parent: Option<HeapNode>,
    pub children: Vec<HeapNode>,
    pub delta_u: Option<DeltaU>,
    pub h: f32,
    pub steady_state_results: Option<Result<SteadyStateResults, SteadyStateError>>,
    pub transient_results: Option<Result<TransientResults, TransientError>>,
    pub contribution: Vec<Contribution>,
    pub depth: usize,
    pub objective: f32,
}
//  impl Deref for AStarNode {


//      fn deref(&self) -> &Self {
//          self
//      }
// }

// impl DerefMut for AStarNode {
//     fn deref_mut(&mut self) -> &mut Self {
//         self
//     }
// }

impl Eq for AStarNode {

}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other.objective.total_cmp(&self.objective)
    }

    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        std::cmp::max_by(self, other, Ord::cmp)
    }

    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        std::cmp::min_by(self, other, Ord::cmp)
    }

    fn clamp(self, _min: Self, _max: Self) -> Self
    where
        Self: Sized,
        Self: PartialOrd,
    {
        todo!()
    }

}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl AStarNode{
    pub fn new(parent: Option<HeapNode>, delta_u: Option<DeltaU>, h: f32) -> Self {
        return AStarNode {
            state: NodeState::Init,
            parent: parent.clone(),
            children: vec![],
            delta_u,
            h,
            steady_state_results: None,
            transient_results: None,
            contribution: vec![Contribution{contri_type:ContributionType::Other,reason:String::from("H"),amount:h }],
            depth: parent.map_or(0, |par| par.borrow().depth + 1),
            objective: h,
        };
    }

    pub fn get_nodes(node: &HeapNode) -> Vec<HeapNode>{
        let mut ret_val: Vec<HeapNode> = vec![node.clone()];

        Self::node_parent_visitor(node, |n| ret_val.push(n.clone()));

        return ret_val.iter().rev().map(|nod| nod.clone()).collect();
    }

    pub fn get_delta_u(node: &HeapNode) -> Vec<DeltaU>{
        let mut ret_val: Vec<DeltaU> = vec![];
        
        Self::node_parent_visitor(node, |n| ret_val.push(n.borrow().delta_u.clone().unwrap()));

        ret_val
    }

    pub fn add_steady_state(&mut self,  contris: Vec<Contribution>, steady_state_results: Result<SteadyStateResults, SteadyStateError>) {
        assert!(self.state == NodeState::Init);
        self.steady_state_results = Some(steady_state_results);

        contris.iter().for_each(|con| {
            assert!(con.contri_type == ContributionType::SteadyState);
            self.contribution.push(con.clone());
        });

        self.state = NodeState::SteadyStateCalculated;
    }

    pub fn add_transient(&mut self, contris: Vec<Contribution>, transient_results: Result<TransientResults, TransientError>) {
        assert!(self.state == NodeState::SteadyStateCalculated);
        self.transient_results = Some(transient_results);

        contris.iter().for_each(|con| {
            assert!(con.contri_type == ContributionType::Transient);
            self.contribution.push(con.clone());
        });

        self.state = NodeState::TransientCalculated;
    }

    fn add_contributions(&mut self, contributions: Vec<Contribution>){
        contributions.iter().for_each(|con| {
            assert!(con.contri_type == ContributionType::SteadyState);
            self.contribution.push(con.clone());
        });
    }

    pub fn node_child_visitor<F>(node: &HeapNode, f: &mut F) where F: FnMut(&Rc<RefCell<AStarNode>>){
        f(node);

        node.borrow().children.iter().for_each(|val| Self::node_child_visitor(val, f));    
    }

    pub fn node_parent_visitor<F>(node: &HeapNode, mut f: F) where F: FnMut(Rc<RefCell<AStarNode>>){
        f(node.clone());
        
        let parent = node.borrow().parent.clone();
        match parent {
            Some(par) => {
                Self::node_parent_visitor(&par, f);
            },
            None => {},
        }        
    }

}

    






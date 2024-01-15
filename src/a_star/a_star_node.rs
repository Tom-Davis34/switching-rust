use std::{rc::Rc, cmp::Ordering};

use crate::power_system::{DeltaU, SigmAlg, plague_algo::PowerFlowNode, U};

use super::steady_state_adapter::{SteadyStateResults, SteadyStateError};

#[derive(Debug, PartialEq,Clone)]
pub enum ContributionType{
    Other,
    SteadyState,
    Tranisent
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
    pub parent: Option<Rc<AStarNode>>,
    pub children: Vec<Rc<AStarNode>>,
    pub delta_u: Option<DeltaU>,
    pub h: f32,
    pub steady_state_results: Option<Result<SteadyStateResults, SteadyStateError>>,
    pub contribution: Vec<Contribution>,
    pub depth: usize,
    pub objective: f32,
}

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

    fn clamp(self, min: Self, max: Self) -> Self
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
    pub fn new(parent: Option<Rc<AStarNode>>, delta_u: Option<DeltaU>, h: f32) -> Self {
        return AStarNode {
            state: NodeState::Init,
            parent: parent.clone(),
            children: vec![],
            delta_u,
            h,
            steady_state_results: None,
            contribution: vec![Contribution{contri_type:ContributionType::Other,reason:String::from("H"),amount:h }],
            depth: parent.map_or(0, |par| par.depth + 1),
            objective: h,
        };
    }

    pub fn get_nodes(self: Rc<Self>) -> Vec<Rc<AStarNode>>{

        let mut ret_val: Vec<Rc<AStarNode>> = vec![self.clone()];
        let mut last = self;

        loop {
            match &last.parent {
                Some(par) => {
                    ret_val.push(par.clone());
                    last = par.clone()
                },
                None => return ret_val.iter().rev().map(|rc| rc.clone()).collect(),
            }
        }
    }
    
    pub fn get_delta_u(&self) -> Vec<DeltaU>{

        let mut ret_val: Vec<DeltaU> = vec![];
        match &self.delta_u {
            Some(delta_u) =>  ret_val.push(delta_u.clone()),
            None => {},
        }

        let mut last = self;

        loop {
            match &last.parent {
                Some(par) => {
                    ret_val.push(par.delta_u.clone().unwrap());
                    last = &par
                },
                None => return ret_val.iter().rev().map(|rc| rc.clone()).collect(),
            }
        }
    }

    pub fn add_steady_state(&mut self,  contribution: Vec<Contribution>, steady_state_results: Result<SteadyStateResults, SteadyStateError>) {
        self.steady_state_results = Some(steady_state_results);
    

    }

    fn add_contributions(&mut self, contributions: Vec<Contribution>){
        contributions.iter().for_each(|con| self.add_contribution(con))
    }

    fn add_contribution(&mut self, contribution: Contribution){
            
    }
}





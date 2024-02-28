use std::{
    borrow::BorrowMut,
    cell::RefCell,
    cmp::Ordering,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::power_system::{DeltaU, PowerSystem, U};

use super::{
    steady_state_adapter::{SteadyStateContri, SteadyStateError, SteadyStateResults},
    transient_adapter::{TransientAdapter, TransientError, TransientResults},
};

pub type HeapNode = Rc<RefCell<AStarNode>>;

#[derive(Debug, PartialEq, Clone)]
pub enum ContributionType {
    Other,
    SteadyState,
    Transient,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Contribution {
    pub contri_type: ContributionType,
    pub reason: String,
    pub amount: f32,
}

#[derive(Debug, PartialEq, Clone)]
pub enum NodeState {
    Init,
    SteadyStateCalculated,
    TransientCalculated,
    Finished,
}

#[derive(Debug, Clone)]
pub struct AStarNode {
    pub display: String,
    pub state: NodeState,
    pub parent: Option<HeapNode>,
    pub children: Vec<HeapNode>,
    pub delta_u: Option<DeltaU>,
    pub h: f32,
    pub steady_state_contri: Option<SteadyStateContri>,
    pub transient_contri: Option<TransientAdapter>,
    pub contribution: Vec<Contribution>,
    pub depth: usize,
    pub objective: f32,
}

impl PartialEq for AStarNode {
    fn eq(&self, other: &Self) -> bool {
        self.delta_u == other.delta_u && self.depth == other.depth
    }
}

impl Eq for AStarNode {}

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

impl AStarNode {
    fn create_display(delta_u: &Option<DeltaU>, ps: &PowerSystem) -> String {
        match delta_u {
            Some(u) => {
                format!("{:?} {}", u.new_u, ps.edges[u.index].name)
            }
            None => "START".to_string(),
        }
    }

    pub fn new(
        parent: Option<HeapNode>,
        delta_u: Option<DeltaU>,
        h: f32,
        ps: &PowerSystem,
    ) -> Self {
        return AStarNode {
            display: Self::create_display(&delta_u, ps),
            state: NodeState::Init,
            parent: parent.clone(),
            children: vec![],
            delta_u,
            h,
            steady_state_contri: None,
            transient_contri: None,
            contribution: vec![Contribution {
                contri_type: ContributionType::Other,
                reason: String::from("H"),
                amount: h,
            }],
            depth: parent.map_or(0, |par| par.borrow().depth + 1),
            objective: h,
        };
    }

    pub fn get_nodes(node: &HeapNode) -> Vec<HeapNode> {
        let mut ret_val: Vec<HeapNode> = vec![node.clone()];

        Self::node_parent_visitor(node, |n| ret_val.push(n.clone()));

        return ret_val.iter().rev().map(|nod| nod.clone()).collect();
    }

    pub fn get_delta_u(node: &HeapNode) -> Vec<DeltaU> {
        let mut ret_val: Vec<DeltaU> = vec![];

        Self::node_parent_visitor(node, |n| match &n.borrow().delta_u {
            Some(du) => ret_val.push(du.clone()),
            None => {}
        });

        ret_val
    }

    pub fn add_steady_state(&mut self, contri: SteadyStateContri) {
        assert!(self.state == NodeState::Init);

        contri.contri.iter().for_each(|con| {
            assert!(con.contri_type == ContributionType::SteadyState);
            self.contribution.push(con.clone());
        });

        self.steady_state_contri = Some(contri);
        self.state = NodeState::SteadyStateCalculated;
    }

    pub fn add_transient(&mut self, contris: TransientAdapter) {
        assert!(self.state == NodeState::SteadyStateCalculated);

        contris.contri.iter().for_each(|con| {
            assert!(con.contri_type == ContributionType::Transient);
            self.contribution.push(con.clone());
        });

        self.transient_contri = Some(contris);
        self.state = NodeState::TransientCalculated;
    }

    fn add_contributions(&mut self, contributions: Vec<Contribution>) {
        contributions.iter().for_each(|con| {
            assert!(con.contri_type == ContributionType::SteadyState);
            self.contribution.push(con.clone());
        });
    }

    pub fn node_child_visitor<F>(node: &HeapNode, f: &mut F)
    where
        F: FnMut(&Rc<RefCell<AStarNode>>),
    {
        f(node);

        node.borrow()
            .children
            .iter()
            .for_each(|val| Self::node_child_visitor(val, f));
    }

    pub fn node_parent_visitor<F>(node: &HeapNode, mut f: F)
    where
        F: FnMut(Rc<RefCell<AStarNode>>),
    {
        f(node.clone());

        let parent = node.borrow().parent.clone();
        match parent {
            Some(par) => {
                Self::node_parent_visitor(&par, f);
            }
            None => {}
        }
    }
}

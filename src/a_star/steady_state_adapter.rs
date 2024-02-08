use std::{error::Error, rc::Rc, time::Duration};

use crate::a_star::a_star_node::Contribution;
use crate::a_star::a_star_node::ContributionType;
use crate::power_system::{plague_algo::PowerFlowNode, PowerSystem, SigmAlg, U};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SteadyStateError {
    Msg(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct SteadyStateResults {
    pub duration: Duration,
    // pub super_node_map: Option<SigmAlg<f32>>,
    pub result: Vec<Rc<PowerFlowNode>>,
}

pub fn compute_ss_contri(
    ps: &PowerSystem,
    u: &Vec<U>,
) -> (
    Vec<Contribution>,
    Result<SteadyStateResults, SteadyStateError>,
) {
    let results = perform_steady_state(ps, u);
    let contri = compute_contri(ps, &results);

    return (contri, results);
}

fn compute_contri(
    ps: &PowerSystem,
    results: &Result<SteadyStateResults, SteadyStateError>,
) -> Vec<Contribution> {
    return vec![Contribution {
        contri_type: ContributionType::SteadyState,
        reason: "test".to_string(),
        amount: 0.0,
    }];
}

fn perform_steady_state(
    ps: &PowerSystem,
    u: &Vec<U>,
) -> Result<SteadyStateResults, SteadyStateError> {
    return Err(SteadyStateError::Msg("Not Implemented".to_string()));
}

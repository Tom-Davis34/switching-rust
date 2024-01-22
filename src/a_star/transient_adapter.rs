use std::{error::Error, rc::Rc, time::Duration};

use crate::a_star::a_star_node::Contribution;
use crate::a_star::a_star_node::ContributionType;
use crate::dop_shared::Stats;
use crate::power_system::{plague_algo::PowerFlowNode, PowerSystem, SigmAlg, U};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TransientError {
    Msg(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct TransientResults {
    stats: Stats,
    t: Vec<f64>,
    out: Vec<Vec<f64>>,
}

pub fn compute_transient_contri(
    ps: &PowerSystem,
    u: &Vec<U>,
) -> (
    Vec<Contribution>,
    Result<TransientResults, TransientError>,
) {
    let results = perform_transient(ps, u);
    let contri = compute_contri(ps, &results);

    return (contri, results);
}

fn compute_contri(
    ps: &PowerSystem,
    results: &Result<TransientResults, TransientError>,
) -> Vec<Contribution> {
    return vec![Contribution {
        contri_type: ContributionType::Transient,
        reason: "transient-test".to_string(),
        amount: 0.0,
    }];
}

fn perform_transient(
    ps: &PowerSystem,
    u: &Vec<U>,
) -> Result<TransientResults, TransientError> {
    return Err(TransientError::Msg("Not Implemented".to_string()));
}

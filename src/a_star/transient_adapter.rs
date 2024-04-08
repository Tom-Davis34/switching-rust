use std::fmt::Display;
use std::{error::Error, rc::Rc};

use chrono::Duration;
use chrono::Utc;

use crate::a_star::a_star_node::Contribution;
use crate::a_star::a_star_node::ContributionType;
use crate::foodes::Stats;
use crate::power_system::{PowerSystem, U};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TransientError {
    Msg(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct TransientResults {
    pub stats: Stats,
    pub t: Vec<f64>,
    pub out: Vec<Vec<f64>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TransientAdapter {
    pub duration: Duration,
    pub contri: Vec<Contribution>,
    pub result: Result<TransientResults, TransientError>,
}

pub fn compute_transient_contri(
    ps: &PowerSystem,
    u: &Vec<U>,
) -> TransientAdapter {
    let start_time = Utc::now();
    let result = perform_transient(ps, u);
    let contri = compute_contri(ps, &result);
    let duration = Utc::now().signed_duration_since(start_time);

    return TransientAdapter {
        duration,
        contri,
        result,
    };
}

fn compute_contri(
    _ps: &PowerSystem,
    _results: &Result<TransientResults, TransientError>,
) -> Vec<Contribution> {
    return vec![Contribution {
        contri_type: ContributionType::Transient,
        reason: "transient-test".to_string(),
        amount: 0.0,
    }];
}

fn perform_transient(
    _ps: &PowerSystem,
    _u: &Vec<U>,
) -> Result<TransientResults, TransientError> {
    return Err(TransientError::Msg("Not Implemented".to_string()));
}

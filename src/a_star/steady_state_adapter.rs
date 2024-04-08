use std::{error::Error, rc::Rc};

use chrono::Duration;
use chrono::Utc;

use crate::a_star::a_star_node::Contribution;
use crate::a_star::a_star_node::ContributionType;
use crate::power_system::power_flow_model::PfNode;
use crate::power_system::{PowerSystem, U};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SteadyStateError {
    Msg(String),
}

#[derive(Debug, Clone)]
pub struct SteadyStateResults {
    pub result: Vec<Rc<PfNode>>,
}

#[derive(Debug, Clone)]
pub struct SteadyStateContri {
    pub duration: Duration,
    pub contri: Vec<Contribution>,
    pub results: Result<SteadyStateResults, SteadyStateError>,
}

pub fn compute_ss_contri(ps: &PowerSystem, u: &Vec<U>) -> SteadyStateContri {
    let start_time = Utc::now();
    let results = perform_steady_state(ps, u);
    let contri = compute_contri(ps, &results);
    let duration = Utc::now().signed_duration_since(start_time);

    return SteadyStateContri {
        duration,
        contri,
        results,
    };
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

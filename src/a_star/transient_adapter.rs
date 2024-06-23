use std::fmt::Display;
use std::{error::Error, rc::Rc};

use chrono::Duration;
use chrono::Utc;

use crate::a_star::a_star_node::Contribution;
use crate::a_star::a_star_node::ContributionType;
use crate::foodes::Stats;
use crate::foodes::foode::State;
use crate::power_system::{PowerSystem, U};

const ERROR_CONTRI: f32 = 10000.0;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TransientError {
    Msg(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct TransientSolution {
    pub stats: Stats,
    pub t: Vec<f32>,
    pub out: Vec<State>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TransientContri {
    pub duration: Duration,
    pub contri: Vec<Contribution>,
    pub result: Result<TransientSolution, TransientError>,
}

pub fn compute_transient_contri(
    ps: &PowerSystem,
    u: &Vec<U>,
) -> TransientContri {
    let start_time = Utc::now();
    // let result: Result<TransientSolution, TransientError> = perform_transient(ps, u);
    let result: Result<TransientSolution, TransientError> = Err(TransientError::Msg("sdf".to_string()));
    let contri = create_tranient_contri(ps, &result);
    let duration = Utc::now().signed_duration_since(start_time);

    return TransientContri {
        duration,
        contri,
        result,
    };
}


fn create_tranient_contri(
    ps: &PowerSystem,
    results: &Result<TransientSolution, TransientError>,
) -> Vec<Contribution> {
    match results {
        Ok(soln) => compute_contri(ps, soln),
        Err(err) => error_contri(err),
    }
}

fn compute_contri(
    ps: &PowerSystem,
    results: &TransientSolution,
) -> Vec<Contribution> {
    let fns: Vec<
        fn(
            &PowerSystem,
            &TransientSolution,
        ) -> Vec<Contribution>,
    > = vec![];

    fns.iter()
        .flat_map(|f| f(ps, results).iter().map(|c| c.clone()).collect::<Vec<Contribution>>())
        .collect::<Vec<Contribution>>()
}

fn error_contri(error: &TransientError) -> Vec<Contribution> {
    return vec![Contribution {
        contri_type: ContributionType::Transient,
        reason: format!("Transient Pf failed: {:?}", error),
        amount: ERROR_CONTRI,
    }];
}
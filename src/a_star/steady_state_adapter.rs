use std::{error::Error, rc::Rc};

use chrono::Duration;
use chrono::Utc;
use num_traits::Zero;

use crate::a_star::a_star_node::Contribution;
use crate::a_star::a_star_node::ContributionType;
use crate::power_system::DeltaU;
use crate::power_system::{PowerSystem, U};
use crate::steady_state;
use crate::steady_state::SteadyStateError;
use crate::steady_state::SteadyStateResults;

use super::a_star_node::HeapNode;

const ERROR_CONTRI: f32 = 10000.0;
const MIN_VOLTAGE: f32 = 0.95;
const MAX_VOLTAGE: f32 = 1.05;

fn disconnectors(
    ps: &PowerSystem,
    results: &SteadyStateResults,
    delta_u: &Option<DeltaU>,
    _previous: &Option<HeapNode>,
) -> Vec<Contribution> {
    match delta_u {
        None => vec![],
        Some(du) => {
            let edge = ps.get_edge(du.index);
            let same_super_node = results.sub_graph_map.get_sub_node(edge.info.fnode)
                == results.sub_graph_map.get_sub_node(edge.info.tnode);

            if same_super_node {
                return vec![];
            }

            let tnode_dead = results.super_v[edge.info.tnode.0].map_or(true, |v| v.is_zero());
            let fnode_dead = results.super_v[edge.info.fnode.0].map_or(true, |v| v.is_zero());

            if tnode_dead || fnode_dead {
                return vec![];
            }

            return vec![Contribution {
                contri_type: ContributionType::SteadyState,
                reason: format!(
                    "Trying to open/close disconnector {}",
                    ps.get_edge(du.index).data.name
                ),
                amount: ERROR_CONTRI,
            }];
        }
    }
}

fn voltage(
    _ps: &PowerSystem,
    results: &SteadyStateResults,
    _delta_u: &Option<DeltaU>,
    _previous: &Option<HeapNode>,
) -> Vec<Contribution> {
    results.super_v.iter().enumerate().map(|(index,opt_v)| {
        opt_v.map(|v| {
            if v.norm() < MIN_VOLTAGE {
                Some(Contribution {
                    contri_type: ContributionType::SteadyState,
                    reason: format!(
                        "Voltage low on node {}",
                        index
                    ),
                    amount: ERROR_CONTRI,
                })
            } else if v.norm() > MAX_VOLTAGE {
                Some(Contribution {
                    contri_type: ContributionType::SteadyState,
                    reason: format!(
                        "Voltage high on node {}",
                        index
                    ),
                    amount: ERROR_CONTRI,
                })
            } else {
                None
            }
        })
    })
    .map(|opt| opt.flatten())
    .filter(|opt| opt.is_some())
    .map(|opt| opt.unwrap())
    .collect()
}

fn blackout(
    ps: &PowerSystem,
    results: &SteadyStateResults,
    _delta_u: &Option<DeltaU>,
    _previous: &Option<HeapNode>,
) -> Vec<Contribution> {
    ps.ps_node_iter().enumerate().filter_map(|(index, ps_node)| {
        if !ps_node.load.is_zero() && results.super_v[index].map_or(true, |v| v.is_zero()) {
            Some(Contribution {
                contri_type: ContributionType::SteadyState,
                reason: format!(
                    "{:#?}MW of load disconnected at node {}", 
                    ps_node.load.re,
                    index
                ),
                amount: ERROR_CONTRI,
            })
        } else {
            None
        }
    }).collect()
}

#[derive(Debug)]
pub struct SteadyStateContri {
    pub duration: Duration,
    pub contri: Vec<Contribution>,
    pub results: Result<SteadyStateResults, SteadyStateError>,
}

impl SteadyStateContri {
    pub fn new(
        duration: Duration,
        contri: Vec<Contribution>,
        results: Result<SteadyStateResults, SteadyStateError>,
    ) -> Self {
        Self {
            duration,
            contri,
            results,
        }
    }
}

pub fn compute_ss_contri(
    ps: &PowerSystem,
    u_vec: &Vec<U>,
    delta_u: &Option<DeltaU>,
    parent: &Option<HeapNode>,
) -> SteadyStateContri {
    let start_time = Utc::now();
    let results = steady_state::steady_state_pf(ps, u_vec);

    let contri = match &results {
        Ok(ss_results) => compute_contri(ps, &ss_results, delta_u, parent),
        Err(error) => error_contri(error.clone()),
    };
    let duration = Utc::now().signed_duration_since(start_time);

    return SteadyStateContri {
        duration,
        contri,
        results,
    };
}

fn compute_contri(
    ps: &PowerSystem,
    results: &SteadyStateResults,
    delta_u: &Option<DeltaU>,
    previous: &Option<HeapNode>,
) -> Vec<Contribution> {
    let fns: Vec<
        fn(
            &PowerSystem,
            &SteadyStateResults,
            &Option<DeltaU>,
            &Option<HeapNode>,
        ) -> Vec<Contribution>,
    > = vec![disconnectors, voltage, blackout];

    fns.iter()
        .flat_map(|f| f(ps, results, delta_u, previous).iter().map(|c| c.clone()).collect::<Vec<Contribution>>())
        .collect::<Vec<Contribution>>()
}

fn error_contri(error: SteadyStateError) -> Vec<Contribution> {
    return vec![Contribution {
        contri_type: ContributionType::SteadyState,
        reason: format!("Steady State Pf failed: {:?}", error),
        amount: ERROR_CONTRI,
    }];
}

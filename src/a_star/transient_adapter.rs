

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TranisentError {
    Msg(String),
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
        contri_type: ContributionType::TranisentError,
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
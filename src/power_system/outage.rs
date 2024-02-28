use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Display,
    iter::{self, zip},
    rc::Rc,
};

use super::{
    plague_algo::{self, SigAlg, SigBasis, SimpleBasisEle, SimpleSigAlg},
    DeltaU, Edge, EdgeData, EdgeIndex, EdgePsNode, NodeIndex, PowerSystem, PsNode, U,
};

#[derive(Debug, Clone)]
pub struct Outage {
    pub in_outage: Vec<bool>,
    pub basis: Vec<Rc<SimpleBasisEle>>,
    pub edges_boundary: Vec<Edge>,
    pub edges_inside: Vec<Edge>,
    pub delta_u: Vec<DeltaU>,
    pub target_u: Vec<U>,
}

#[derive(Debug)]
pub struct GenerateOutageError {
    names_failed: Vec<String>,
}

impl Display for GenerateOutageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "Unable to determine the edges for names: {:?}",
            self.names_failed
        );
    }
}

impl Error for GenerateOutageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}



pub fn generate_outage(
    ps: &PowerSystem,
    edge_names: Vec<String>,
) -> Result<Outage, GenerateOutageError> {
    
    let edges = match get_edge_from_edge_names(edge_names, ps) {
        Ok(value) => value,
        Err(value) => return Err(value),
    };

    let basis_eles_dups = edges
        .iter()
        .flat_map(|f| vec![f.tbus.index, f.fbus.index])
        .map(|node_index| ps.sigma.to_basis.get(node_index).unwrap().index)
        .collect::<HashSet<usize>>();

    let basis_eles = basis_eles_dups
        .iter()
        .map(|i| ps.sigma.basis.get(*i).unwrap().clone())
        .collect::<Vec<Rc<SimpleBasisEle>>>();

    let outage_nodes = basis_eles
        .iter()
        .flat_map(|be| be.nodes().iter().map(|n| n.index))
        .collect::<Vec<NodeIndex>>();

    let mut edges_boundary = Vec::new();
    let mut edges_inside = Vec::new();

    let target_u = zip(ps.switch_iter(), ps.start_u.iter())
        .map(|(e, u)| {
            if outage_nodes.contains(&e.fbus.index) != outage_nodes.contains(&e.tbus.index) {
                edges_boundary.push(e.clone());
                return U::Open;
            } else if outage_nodes.contains(&e.fbus.index) {
                edges_inside.push(e.clone());
                return U::DontCare;
            } else {
                return u.clone();
            }
        })
        .collect::<Vec<U>>();

    let delta_u = zip(target_u.iter(), ps.start_u.iter())
        .enumerate()
        .filter(|(_i, (tu, su))| tu != su)
        .map(|(index, (tu, _su))| DeltaU {
            index: index,
            new_u: *tu,
        })
        .collect::<Vec<DeltaU>>();

    Ok(Outage {
        in_outage: ps
            .nodes_iter()
            .map(|n| outage_nodes.contains(&n.index))
            .collect(),
        basis: basis_eles,
        edges_boundary: edges_boundary,
        edges_inside: edges_inside,
        delta_u: delta_u,
        target_u: target_u,
    })
}

fn get_edge_from_edge_names(
    edge_names: Vec<String>,
    ps: &PowerSystem,
) -> Result<Vec<Rc<Edge>>, GenerateOutageError> {
    let edges_opt = edge_names
        .iter()
        .map(|en| ps.get_edge_by_name(en))
        .collect::<Vec<Option<&Rc<Edge>>>>();

    let errs = zip(&edge_names, &edges_opt)
        .filter(|(_name, opt_index)| opt_index.is_none())
        .map(|tu| tu.0.to_owned())
        .collect::<Vec<String>>();

    if errs.len() > 0 {
        return Err(GenerateOutageError { names_failed: errs });
    }

    let edges = edges_opt
        .into_iter()
        .map(Option::unwrap)
        .map(Rc::clone)
        .collect::<Vec<Rc<Edge>>>();

    Ok(edges)
}
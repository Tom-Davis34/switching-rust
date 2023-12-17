use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;
use std::fmt::Debug;

use crate::traits::C32;
mod file_parsing;

type U = bool;

#[derive(Debug, Clone, Copy)]
pub enum NodeType {
	GND, PQ, PV, Sk 
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError;

#[derive(Debug, Clone)]
pub struct PsNode{
    pub num: usize,
    pub load: C32,
    pub gen: C32,
    pub n_type: NodeType,

    pub sws: Vec<Rc<Switch>>,
    pub cirs: Vec<Rc<Circuit>>,
}


#[derive(Debug, Clone)]
pub struct Switch{
    pub is_cb: bool,
    pub fbus: Rc<PsNode>,
    pub tbus: Rc<PsNode>,
}


#[derive(Debug, Clone)]
pub struct Circuit {
    pub fbus: Rc<PsNode>,
    pub tbus: Rc<PsNode>,
    pub admittance: C32,
    pub line_charge: f32
}


#[derive(Debug, Clone)]
pub struct PowerSystem {
    pub nodes: Vec<PsNode>,
    pub sws: Vec<Switch>,
    pub cirs: Vec<Circuit>,

    pub start_u: Vec<U>,

    // pub nodes_rc: Vec<Rc<PsNode>>,
    // pub sws_rc: Vec<Rc<Switch>>,
    // pub cirs_rc: Vec<Rc<Circuit>>,
}


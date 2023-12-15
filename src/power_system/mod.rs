use std::str::FromStr;
use std::fmt::Debug;

use crate::traits::C32;
mod file_parsing;


pub trait FromRow: Sized {
    type Err;
    fn from_str(s: &str, num: usize) -> Result<Self, Self::Err>;
}

pub enum NodeType {
	GND, PQ, PV, Sk 
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError;

pub struct PsNode<'g>{
    pub num: usize,
    pub load: C32,
    pub gen: C32,
    pub n_type: NodeType,
    pub line_charge: f32,

    pub sws: Vec<&'g Switch<'g>>,
    pub cirs: Vec<&'g Circuit<'g>>,
}

pub struct Switch<'g>{
    pub is_cb: bool,
    pub fbus: &'g PsNode<'g>,
    pub tbus: &'g PsNode<'g>,
}

pub struct Circuit<'g> {
    pub fbus: &'g PsNode<'g>,
    pub tbus: &'g PsNode<'g>,
    pub admittance: C32,
    pub line_charge: f32
}

pub struct PowerSystem<'g> {
    pub nodes: Vec<PsNode<'g>>,
    pub sws: Vec<Switch<'g>>,
    pub cirs: Vec<Circuit<'g>>,
}


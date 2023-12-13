use std::str::FromStr;
use std::fmt::Debug;

use crate::traits::C32;


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
    pub num: usize,
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

fn split_whitespace(s: &str) -> Vec<&str>{
    s.split_whitespace().collect::<Vec<&str>>()
}


fn parse<T>(cells: &Vec<&str>, i: usize) -> T where T: FromStr, <T as FromStr>::Err: Debug{
    cells.get(i).unwrap().parse::<T>().unwrap()
}

impl<'g> Circuit<'g>{
    
    fn from_str(s: &str, num: usize, nodes: &'g Vec<&'g PsNode>) -> Result<Self, ParseError> {
        let cells = split_whitespace(s);


        let fbus_index = parse::<usize>(&cells, 0);
        let tbus_index = parse::<usize>(&cells, 1);

        let cir = Circuit {
            num: num,
            fbus: nodes.iter().find(|n| n.num == fbus_index).unwrap(),
            tbus: nodes.iter().find(|n| n.num == tbus_index).expect("Cannot find tbus"),
            admittance: C32::new(1.0, 0.0) / C32::new(parse::<f32>(&cells, 2), parse::<f32>(&cells, 3)),
            line_charge: parse::<f32>(&cells, 4),
        };

        Ok(cir)
    }


}
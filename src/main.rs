#![allow(dead_code)]
#![allow(unused_imports)]
#![warn(incomplete_features)]
#![feature(generic_const_exprs)]

pub mod matrix_builder;
pub mod runge_kutta;
pub mod traits;
pub mod foode;
pub mod matrix_utils;
pub mod dop_shared;
pub mod dop853;
pub mod dopri5;
pub mod butcher_tableau;
pub mod controller;
pub mod rk4;
pub mod power_system;
pub mod steady_state_pf;

use dop_shared::System;
use foode::{State, Time};

use crate::dop853::*;

fn main() {

}

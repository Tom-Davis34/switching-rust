#![allow(dead_code)]
#![allow(unused_imports)]
#![warn(incomplete_features)]
// #![feature(generic_const_exprs)]

use clap::{command, Parser};
use power_system::PowerSystem;

use crate::{power_system::*, a_star::{a_star_node::AStarNode, AStar}};

pub mod matrix_builder;
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
pub mod a_star;
pub mod utils;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("default"))]
    name: String,

    #[arg(short, long, default_value_t = String::from("./grids/BRB/"))]
    ps: String,

    #[arg(short, long)]
    outage: String,
}   

fn main() {
    let args = Args::parse();
    println!("{:#?}", args);

    let outage_strs = args.outage.split(",").map(|s| s.to_string()).collect::<Vec<String>>();

    let ps = PowerSystem::from_files(&args.ps);
    println!("PS: {:#?}", &ps);

    let outage_res = power_system::plague_algo::generate_outage(&ps, outage_strs);

    match outage_res {
        Ok(outage) => {
            println!("outage: {:#?}", &outage);
            run_astar(&ps, &outage);
        },
        Err(err) => panic!("Could not generate outage. Error: {}", err)
    }
}

fn run_astar(ps: &PowerSystem, outage: &Outage) -> AStar{
    let astar = AStar::new();
    let astar_result =  astar.run(ps, outage);
    // println!("{:#?}", ps);
    // println!("{:#?}", outage);
    println!("{}", astar_result.stats);
    match &astar_result.os {
        Some(os) => println!("OS:\n{}", os),
        None => todo!(),
    }
    

    return astar_result;
}

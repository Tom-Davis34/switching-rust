#![allow(dead_code)]
#![allow(unused_imports)]
#![warn(incomplete_features)]
// #![feature(generic_const_exprs)]

use clap::{command, Parser};
use power_system::{PowerSystem, outage::Outage};

use crate::{power_system::*, a_star::{a_star_node::AStarNode, AStar}};

pub mod matrix_builder;
pub mod traits;
pub mod foodes;
pub mod matrix_utils;
pub mod power_system;
pub mod a_star;
pub mod utils;
pub mod steady_state;
pub mod graph;


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

    let outage_res = power_system::outage::generate_outage(&ps, outage_strs);

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
    let astar_result =  astar.run_generate(ps, outage);
    // println!("{:#?}", ps);
    // println!("{:#?}", outage);
    println!("{}", astar_result.stats);
    match &astar_result.os {
        Some(os) => println!("OS:\n{}", os),
        None => panic!(),
    }

    return astar_result;
}

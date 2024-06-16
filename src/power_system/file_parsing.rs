use std::{fs, string};

use super::*;

const FILE_NAME_GENS: &str = "Gens.txt";
const FILE_NAME_CIRCUITS: &str = "Circuits.txt";
const FILE_NAME_SWITCHES: &str = "Switches.txt";
const FILE_NAME_BUSES: &str = "Buses.txt";

pub fn parse_ps(path: &str) -> FileParseResults {
    let gens_strs = read_file(path, FILE_NAME_GENS);
    let gens: Vec<Gen> = gens_strs.iter().map(|s| Gen::from_row(&s)).collect();

    let bus_strs = read_file(path, FILE_NAME_BUSES);

    let start_u = parse_start_u(path, FILE_NAME_SWITCHES, FILE_NAME_CIRCUITS);

    let ps_nodes: Vec<PsNode> = bus_strs
        .iter()
        .enumerate()
        .map(|s| PsNode::from_row(s, &gens))
        .collect();
    let ps_nodes_rc: Vec<Rc<PsNode>> = ps_nodes.iter().map(|f| Rc::new(f.clone())).collect();

    let switch_strs: Vec<String> = read_file(path, FILE_NAME_SWITCHES);
    let mut switches: Vec<FileEdge> = switch_strs
        .iter()
        .enumerate()
        .map(|s| PsEdge::from_switch_row(s, &ps_nodes_rc, &start_u))
        .collect();
    // let switches_rc: Vec<Rc<Switch>> = switches.iter().map(|f| Rc::new(f.clone())).collect();

    let cicuits_strs = read_file(path, FILE_NAME_CIRCUITS);
    let mut cicuits: Vec<FileEdge> = cicuits_strs
        .iter()
        .enumerate()
        .map(|s| (s.0, s.1) )
        .map(|s| PsEdge::from_circuit_row(s, &ps_nodes_rc, switches.len(), &start_u))
        .collect();
    // let cicuits_rc: Vec<Rc<Circuit>> = cicuits.iter().map(|f| Rc::new(f.clone())).collect();

    switches.append(&mut cicuits);


    FileParseResults {
        nodes: ps_nodes,
        edges: switches,
        start_u: start_u,
    }
}

#[derive(Debug, Clone)]
pub struct FileParseResults {
    pub nodes: Vec<PsNode>,
    pub edges: Vec<FileEdge>,
    pub start_u: Vec<U>,
}

#[derive(Debug, Clone)]
pub(super) struct FileEdge{
    pub(super) edge: PsEdge,
    pub(super) tbus: NodeIndex,
    pub(super) fbus: NodeIndex,
}

#[derive(Debug, Clone, Copy)]
struct Gen {
    bus: usize,
    p: f32,
    q: f32,
}

fn split_whitespace(s: &str) -> Vec<&str> {
    s.split_whitespace().collect::<Vec<&str>>()
}

fn parse<T>(cells: &Vec<&str>, i: usize) -> T
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    // println!("cells {:?}, i {} ", cells, i);
    cells.get(i).unwrap().parse::<T>().unwrap()
}

fn read_file(path: &str, name: &str) -> Vec<String> {
    fs::read_to_string(path.to_owned() + name)
        .expect(&("Cannot find file ".to_owned() + path + name))
        .lines()
        .skip(1)
        .map(|f| f.to_string())
        .collect::<Vec<String>>()
}

fn find_node(num: usize, nodes: &Vec<Rc<PsNode>>) -> Rc<PsNode> {
    nodes
        .iter()
        .find(|n| n.num == num)
        .and_then(|f| Some(f.clone()))
        .expect(&format!("Cannot find node with num = {}", num))
}

fn parse_start_u(path: &str, name_switches: &str, name_circuits: &str) -> Vec<U> {
    let switch_strs = read_file(path, name_switches);
    let circuit_strs = read_file(path, name_circuits);

    let mut res = switch_strs
        .iter()
        .map(|s| {
            split_whitespace(s)
                .get(2)
                .map(|s| s.parse::<i32>().unwrap() == 1)
                .unwrap()
        })
        .map(|b| if b { U::Open } else { U::Closed })
        .collect::<Vec<U>>();

    circuit_strs.iter().for_each(|_str| res.push(U::DontCare));

    return res;
}

impl Gen {
    fn from_row(s: &str) -> Self {
        let cells = split_whitespace(s);

        Gen {
            bus: parse::<usize>(&cells, 0),
            p: parse::<f32>(&cells, 1),
            q: parse::<f32>(&cells, 2),
        }
    }
}

impl PsEdge {
    fn from_circuit_row(s: (usize, &String), nodes: &Vec<Rc<PsNode>>, swicth_num: usize, _start_u: &Vec<U>) -> FileEdge {
        let cells = split_whitespace(s.1);

        let fbus_num = parse::<usize>(&cells, 0);
        let fbus = find_node(fbus_num, nodes);
        let tbus_num = parse::<usize>(&cells, 1);
        let tbus = find_node(tbus_num, nodes);

        let cir = Circuit {
            admittance: C32::new(1.0, 0.0)
                / C32::new(parse::<f32>(&cells, 2), parse::<f32>(&cells, 3)),
            line_charge: parse::<f32>(&cells, 4),
        };


        FileEdge {
            edge: PsEdge {
                index: EdgeIndex(s.0 + swicth_num),
                name: format!("Cir{:?}", s.0),
                u: U::DontCare,
                data: EdgeData::Cir(cir),
            },
            fbus: fbus.index,
            tbus: tbus.index,
        }

    }

    fn from_switch_row(s: (usize, &String), nodes: &Vec<Rc<PsNode>>, start_u: &Vec<U>) -> FileEdge {
        let cells = split_whitespace(s.1);

        let fbus_num = parse::<usize>(&cells, 0);
        let fbus = find_node(fbus_num, nodes);
        let tbus_num = parse::<usize>(&cells, 1);
        let tbus = find_node(tbus_num, nodes);

        let is_cb = parse::<usize>(&cells, 3) == 1;

        let sw = Switch { is_cb: is_cb };

        
        FileEdge {
            edge: PsEdge {
                index: EdgeIndex(s.0),
                name: match is_cb {
                    true => format!("CB{:?}", s.0),
                    false => format!("Dis{:?}", s.0),
                },
                u: start_u[s.0],
                data: EdgeData::Sw(sw),
            },
            fbus: fbus.index,
            tbus: tbus.index,
        }
        
    }
}

fn get_node_type(type_i32: i32) -> NodeType {
    if type_i32 == 1 {
        NodeType::PQ
    } else if type_i32 == 2 {
        NodeType::PV
    } else if type_i32 == 3 {
        NodeType::Sk
    } else {
        panic!("incorrect node type")
    }
}

impl PsNode {
    fn from_row(s: (usize, &String),  gens: &Vec<Gen>) -> Self {
        // %id type Pd	     Qd
        let cells = split_whitespace(s.1);

        let num = parse::<usize>(&cells, 0);

        let type_i32 = parse::<i32>(&cells, 1);
        let nt: NodeType = get_node_type(type_i32);

        let real_load = parse::<f32>(&cells, 2);
        let img_load = parse::<f32>(&cells, 3);
        let load = C32::new(real_load, img_load);

        let gens = gens.iter().find(|f| f.bus == num);
        let real_gen = gens.map_or(0.0, |gen| gen.p);
        let img_gen = gens.map_or(0.0, |gen| gen.q);
        let gen = C32::new(real_gen, img_gen);

        let system_v = parse::<f32>(&cells, 9);

        PsNode {
            index: NodeIndex(s.0),
            num: num,
            load: load,
            gen: gen,
            system_v,
            n_type: nt,
        }
    }
}

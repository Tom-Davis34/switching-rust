use std::{fs, string};

use super::*;

const FILE_PATH: &str = "./grids/BRB/"; 
const FILE_NAME_GENS : &str = "Gens.txt"; 
const FILE_NAME_CIRCUITS : &str = "Circuits.txt"; 
const FILE_NAME_SWITCHES : &str = "Switches.txt"; 
const FILE_NAME_BUSES : &str = "Buses.txt"; 


#[derive(Clone, Copy)]
struct Gen{
    bus: usize,
    p: f32,
    q: f32
}

fn split_whitespace(s: &str) -> Vec<&str>{
    s.split_whitespace().collect::<Vec<&str>>()
}

fn parse<T>(cells: &Vec<&str>, i: usize) -> T where T: FromStr, <T as FromStr>::Err: Debug{
    cells.get(i).unwrap().parse::<T>().unwrap()
}

fn read_file(path: &str, name: &str) -> Vec<String> {
    fs::read_to_string(path.to_owned() + name).expect(&("Cannot find file".to_owned() + name))
            .lines()
            .map(|f| f.to_string())
            .collect::<Vec<String>>()
}

fn find_node(num: usize, nodes: &Vec<Rc<PsNode>>) -> Rc<PsNode>{
    nodes.iter().find(|n| n.num == num).and_then(|f| Some(f.clone())).expect(&format!("Cannot find node with num = {}", num))
}

fn parse_start_u(path: &str, name: &str) -> Vec<U> {
    let switch_strs = read_file(path, name);

    switch_strs.iter().map(|s| split_whitespace(s).get(3).map(|s| s.parse::<i32>().unwrap() == 1).unwrap()).collect()

}



fn parse_ps(path: &str) -> PowerSystem{
    let gens_strs = read_file(path, FILE_NAME_GENS);
    let gens: Vec<Gen> = gens_strs.iter().map(|s| Gen::from_row(&s)).collect();

    let bus_strs = read_file(path, FILE_NAME_BUSES);
    let mut ps_nodes: Vec<PsNode> = bus_strs.iter().map(|s| PsNode::from_row(&s, &gens)).collect();
    let ps_nodes_rc: Vec<Rc<PsNode>> = ps_nodes.iter().map(|f| Rc::new(f.clone())).collect();

    let switch_strs = read_file(path, FILE_NAME_SWITCHES);
    let switches: Vec<Switch> = switch_strs.iter().map(|s| Switch::from_row(&s, &ps_nodes_rc)).collect();
    let switches_rc: Vec<Rc<Switch>> = switches.iter().map(|f| Rc::new(f.clone())).collect();

    let cicuits_strs = read_file(path, FILE_NAME_CIRCUITS);
    let cicuits: Vec<Circuit> = cicuits_strs.iter().map(|s| Circuit::from_row(&s, &ps_nodes_rc)).collect();
    let cicuits_rc: Vec<Rc<Circuit>> = cicuits.iter().map(|f| Rc::new(f.clone())).collect();

    let start_u = parse_start_u(path, FILE_NAME_SWITCHES);

    ps_nodes = assign_edges_to_nodes(ps_nodes, &switches_rc, &cicuits_rc);
    // ps_nodes_rc = assign_edges_to_nodes_rc(ps_nodes_rc, &switches_rc, &cicuits_rc);

    PowerSystem{
        nodes: ps_nodes,
        sws: switches,
        cirs: cicuits,

        start_u: start_u,

        // nodes_rc: ps_nodes_rc,
        // sws_rc: switches_rc,
        // cirs_rc: cicuits_rc,
    }
}

fn assign_edges_to_nodes(mut ps_nodes: Vec<PsNode>, switches_rc: &Vec<Rc<Switch>>, cicuits_rc: &Vec<Rc<Circuit>>) -> Vec<PsNode> {
    
    for ele in ps_nodes.iter_mut() {
        ele.sws = switches_rc.iter().filter(|s| s.fbus.num == ele.num || s.tbus.num == ele.num ).map(|s| Rc::clone(s)).collect();
        ele.cirs = cicuits_rc.iter().filter(|s| s.fbus.num == ele.num || s.tbus.num == ele.num ).map(|s| Rc::clone(s)).collect();
    }

    return ps_nodes;
}

fn assign_edges_to_nodes_rc(mut ps_nodes: Vec<Rc<PsNode>>, switches_rc: &Vec<Rc<Switch>>, cicuits_rc: &Vec<Rc<Circuit>>) -> Vec<Rc<PsNode>> {
    
    for ele in ps_nodes.iter_mut() {
        ele.sws = switches_rc.iter().filter(|s| s.fbus.num == ele.num || s.tbus.num == ele.num ).map(|s| Rc::clone(s)).collect();
        ele.cirs = cicuits_rc.iter().filter(|s| s.fbus.num == ele.num || s.tbus.num == ele.num ).map(|s| Rc::clone(s)).collect();
    }

    return ps_nodes;
}

impl Gen{ 
    fn from_row(s: &str) -> Self{
        let cells = split_whitespace(s);

       Gen {
            bus: parse::<usize>(&cells, 0),
            p: parse::<f32>(&cells, 1),
            q: parse::<f32>(&cells, 2),
        }
    }
}

impl Circuit{
    
    fn from_row(s: &str, nodes: &Vec<Rc<PsNode>>) -> Self {
        let cells = split_whitespace(s);

        let fbus_num = parse::<usize>(&cells, 0);
        let fbus = find_node(fbus_num, nodes);
        let tbus_num = parse::<usize>(&cells, 1);
        let tbus = find_node(tbus_num, nodes);

        let cir = Circuit {
            fbus: fbus,
            tbus: tbus,
            admittance: C32::new(1.0, 0.0) / C32::new(parse::<f32>(&cells, 2), parse::<f32>(&cells, 3)),
            line_charge: parse::<f32>(&cells, 4),
        };

        cir
    }
}

impl Switch{
    fn from_row(s: &str, nodes: &Vec<Rc<PsNode>>) -> Self{
        let cells = split_whitespace(s);


        let fbus_num = parse::<usize>(&cells, 0);
        let fbus = find_node(fbus_num, nodes);
        let tbus_num = parse::<usize>(&cells, 1);
        let tbus = find_node(tbus_num, nodes);

        let is_cb = parse::<usize>(&cells, 3) == 1;

        let sw = Switch {
            fbus: fbus,
            tbus: tbus,
            is_cb: is_cb
        };

        sw
    }
}


fn get_node_type(type_i32: i32) -> NodeType {
    if type_i32 == 1 {
        NodeType::PQ
    } else if type_i32 == 2 {
        NodeType::PV
    } else if  type_i32 == 3 {
        NodeType::Sk
    } else {
        panic!("incorrect node type")
    }
}

impl PsNode{
    fn from_row(s: &str, gens: &Vec<Gen>) -> Self {
        // %id type Pd	     Qd   	 
        let cells = split_whitespace(s);

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

        PsNode { num: num, load: load, gen: gen, n_type: nt, sws: Vec::new(), cirs: Vec::new()}
    }
}
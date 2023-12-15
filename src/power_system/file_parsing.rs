use super::*;

struct Gen{
    bus: i32,
    p: f32,
    q: f32
}

fn split_whitespace(s: &str) -> Vec<&str>{
    s.split_whitespace().collect::<Vec<&str>>()
}

fn parse<T>(cells: &Vec<&str>, i: usize) -> T where T: FromStr, <T as FromStr>::Err: Debug{
    cells.get(i).unwrap().parse::<T>().unwrap()
}

impl Gen{ 
    fn from_row(s: &str) -> Self{
        let cells = split_whitespace(s);

       Gen {
            bus: parse::<i32>(&cells, 0),
            p: parse::<f32>(&cells, 1),
            q: parse::<f32>(&cells, 2),
        }
    }
}

impl<'g> Circuit<'g>{
    
    fn from_row(s: &str, nodes: &'g Vec<&'g PsNode>) -> Self {
        let cells = split_whitespace(s);

        let fbus_index = parse::<usize>(&cells, 0);
        let tbus_index = parse::<usize>(&cells, 1);

        let cir = Circuit {
            fbus: nodes.iter().find(|n| n.num == fbus_index).unwrap(),
            tbus: nodes.iter().find(|n| n.num == tbus_index).expect("Cannot find tbus"),
            admittance: C32::new(1.0, 0.0) / C32::new(parse::<f32>(&cells, 2), parse::<f32>(&cells, 3)),
            line_charge: parse::<f32>(&cells, 4),
        };

        cir
    }
}

impl<'g> Switch<'g>{
    fn from_row(s: &str, nodes: &'g Vec<&'g PsNode>) -> Self{
        let cells = split_whitespace(s);


        let fbus_index = parse::<usize>(&cells, 0);
        let tbus_index = parse::<usize>(&cells, 1);
        let is_cb = parse::<usize>(&cells, 3) == 1;

        let sw = Switch {
            fbus: nodes.iter().find(|n| n.num == fbus_index).unwrap(),
            tbus: nodes.iter().find(|n| n.num == tbus_index).expect("Cannot find tbus"),
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

impl<'g> PsNode<'g>{
    fn from_row(s: &str, gens: Vec<Gen>) -> Self {
        // %id type Pd	     Qd   	 
        let cells = split_whitespace(s);

        let num = parse::<usize>(&cells, 0);

        let type_i32 = parse::<i32>(&cells, 1);
        let mut nt: NodeType = get_node_type(type_i32);

        let real_load = parse::<f32>(&cells, 2);
        let img_load = parse::<f32>(&cells, 3);
        let load = C32::new(real_load, img_load);

        let 

        PsNode { num: num, load: (), gen: (), n_type: (), sws: vec![], cirs: vec![]}
    }
}
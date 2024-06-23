use std::{io::{repeat, Read}, iter::{self}};

use nalgebra_sparse::{CsrMatrix, SparseFormatError};

use crate::{power_system::{PowerSystem, U, DeltaU, PsEdge, PsNode}, traits::C32, matrix_builder::{self, MatBuilder, CsrMatBuilder}, graph::{transform::{CreateSubGraph, SubGraphMap}, Graph, EdgeIndex, NodeIndex}, utils::is_zero, foodes::{dop853::Dop853, foode::{Foode, State, TransientSolve}}};

use super::transient_adapter::{TransientSolution, TransientError};

const RG: f32 = 1.0;
const RGC: f32 = 100.0;
const LG: f32 =  0.00525;
const CG: f32 =  0.000525;

const SWR: f32 =  0.001;

const R_TOLERANCE:f32 = 0.01;
const A_TOLERANCE:f32 = 0.01;
const DX:f32 = 1.0 / 50.0 / 1000.0;

fn create_mat(    
    g: &Graph<PsNode, PsEdge>,
    _u: &Vec<U>,
) -> (CsrMatBuilder<f32>, Vec<f32>) {

    let mut cap_to_gnd = iter::repeat(0.0).take(g.get_node_count()).collect::<Vec<f32>>();

    let voltage_num = g.node_data.len();
	let current_from_gnd_num = voltage_num * 2;
	let circuit_num = g.edge_data.len();

    let index_gen_current = voltage_num;
	let index_load_current = voltage_num*2;
	let index_cir_current = voltage_num*3;

	let total_rows: usize = voltage_num + current_from_gnd_num + circuit_num;

    let mut smb_a = matrix_builder::CsrMatBuilder::<f32>::new(total_rows, g.get_node_count());
    
    //gens...
    for current_index in 0..voltage_num {

            let node_data = &g.node_data[current_index];

            let lg = LG / node_data.gen.re;
            let rg = RG / node_data.gen.re;
            let rgc = RGC / node_data.gen.re;
            let cg = node_data.gen.re * CG;
    
            let index_i = current_index + index_gen_current;
            let index_v = current_index;
    
            smb_a.add(index_i, index_i, -1.0 / lg);
            
            smb_a.add(index_i, current_index, -rg / lg);
    
            smb_a.add(index_v, index_v, -1.0 / rgc);
    
            cap_to_gnd[index_v] += cg;
    
            smb_a.set(index_v, index_i, 1.0);
    }

    //loads...
    for current_index in 0..voltage_num {

            let node_data = &g.node_data[current_index];

            if is_zero(&node_data.load) {
                continue;
            }

            let susceptance = node_data.load.inv();

            let ll = susceptance.im.abs();
            let rl = susceptance.re.abs();
        
            let index_i = current_index + index_load_current;
            let index_v = current_index;
    
            smb_a.add(index_i, index_i, -rl / ll);
            smb_a.add(index_i, index_v, -1.0 / ll);

            smb_a.set(index_v, index_i, 1.0);
    }

    for index in 0..g.edges().len() {

        let edge = g.get_edge(EdgeIndex(index));

        if is_zero(&edge.data.admittance()) {
            continue;
        }

        let impedance = edge.data.admittance().inv();
        
        let r = impedance.re;
		let ind = impedance.im;

		let index_i = index + index_cir_current;
        let f_node = edge.info.fnode.0;
        let t_node = edge.info.tnode.0;

		//i dot
		smb_a.add(index_i, index_i, -r/ind);
		smb_a.add(index_i, f_node, -1.0/ind);
		smb_a.add(index_i, t_node, 1.0/ind);

        cap_to_gnd[f_node] += edge.data.line_charge() / 2.0;
        cap_to_gnd[t_node] += edge.data.line_charge() / 2.0;
    } 

    smb_a.mut_map(|r, _c, ele| {
        if r < voltage_num && cap_to_gnd[r] > 0.0 {
            return ele / cap_to_gnd[r];
        } else {
            return 0.0;
        }
    });

    return (smb_a, cap_to_gnd);
}

fn create_sub_graph(ps: &PowerSystem, u_vec: &Vec<U>, du: &DeltaU) -> (Graph<PsNode, PsEdge>, SubGraphMap){
    
    let live_nodes = ps.live_nodes(u_vec);
    println!("live nodes {:?}",live_nodes);
    let nm = |n: &PsNode| n.clone();
    let nf = |n: &PsNode| live_nodes.contains(&n.index);
    let em = |e: &PsEdge| e.clone();
    let mut subgraph_creator = CreateSubGraph::new(&ps.g, nm, nf, em);

    let edge_contraction_node_merge = |_e: &PsEdge, fnode: &PsNode, tnode: &PsNode | {
        // println!("fnode {:?}", fnode);
        // println!("tnode {:?}", tnode);
        let load = match  fnode.num == tnode.num {
            true => fnode.load,
            false => fnode.load + tnode.load,
        };

        let gen = match  fnode.num == tnode.num {
            true => fnode.gen,
            false => fnode.gen + tnode.gen,
        };

        let res = PsNode {
            num: tnode.num,
            index: tnode.index,
            load: load,
            gen: gen,
            system_v: tnode.system_v,
            n_type: fnode.n_type.max(tnode.n_type),
        };
        return res;
    };

    let edge_contraction_edge_filter = |e: &PsEdge | { e.is_switch() && e.u == U::Closed  && e.index != du.index };

    subgraph_creator.edge_contraction_filter(&edge_contraction_node_merge, &edge_contraction_edge_filter);
    return subgraph_creator.complete();
}

fn add_switch_resistance(smb_a: &mut CsrMatBuilder<f32>, cap_to_gnd: &Vec<f32>,f_node: usize, t_node: usize,) {
    let f_rc = 1.0 / SWR / cap_to_gnd[f_node];
    let t_rc = 1.0 / SWR / cap_to_gnd[t_node];

	smb_a.add(f_node, f_node, -f_rc);
	smb_a.add(f_node, t_node,  f_rc);
	smb_a.add(t_node, t_node, -t_rc);
	smb_a.add(t_node, f_node,  t_rc);
}

fn create_b(    
    g: &Graph<PsNode, PsEdge>,
) -> Vec<f32>{
//totalRows x totalRows
    // let voltage_num = g.node_data.len();
	// let current_from_gnd_num = voltage_num * 2;
	// let circuit_num = g.edge_data.len();

    // let index_gen_current = voltage_num;
	// let index_load_current = voltage_num*2;
	// let index_cir_current = voltage_num*3;

	// let total_rows: usize = voltage_num + current_from_gnd_num + circuit_num;
    
    // let mut smb_b = matrix_builder::CsrMatBuilder::<f32>::new(total_rows, g.get_node_count());


    // for i in 0..voltage_num {
    //     let gen = g.node_data[i].gen;

    //     if is_zero(&gen) {
    //         continue;
    //     }

    //     let lg = LG / gen.re;

    //     let index_i = i + voltage_num;

    //     smb_b.add(index_i, index_i, 1.0/lg);
    // }


    g.node_data.iter().map(|nd| {
        if is_zero(&nd.gen) {
            return 0.0;
        } else {
            return LG / nd.gen.re;
        }
    }).collect()

}

pub fn perform_transient(
    ps: &PowerSystem,
    u: &Vec<U>,
    du: &DeltaU,
) -> Result<TransientSolution, TransientError> {
    let (simplier_graph, sub_graph_map) = create_sub_graph(ps, u, du);

    let (mut mat_b, cap_to_gnd) = create_mat(&simplier_graph, u);

    let closed = mat_b.build().map_err(|_err|
        {return TransientError::Msg("closed sparse format error".to_string())}
    )?;

    let sub_edge_index = sub_graph_map.get_sub_edge(du.index).unwrap();
    let edge = simplier_graph.get_edge(sub_edge_index);
    add_switch_resistance(&mut mat_b, &cap_to_gnd, edge.info.fnode.0, edge.info.tnode.0);

    let open = mat_b.build().map_err(|_err|
        {return TransientError::Msg("open sparse format error".to_string())}
    )?;

    let (a, a_tilder) = match du.new_u {
        U::Open => (closed, open),
        U::Closed => (open, closed),
        U::DontCare => panic!("oh no"),
    };

    let b = create_b(&simplier_graph);

    let x_start = 0.0;
    let x_end = 1.0/50.0 * 4.0;

    let switch_time: f32 = 1.0/50.0 * 2.0;
    let start = State::repeat(a.nnz(), 0.0);

    let system = TransientSolve{
        gen_curr_index: simplier_graph.get_node_count(),
        a,
        a_tilder,
        b: b,
        switch_time,
    };


    let mut dop = Dop853::new(
        system.clone(),
        x_start,
        x_end,
        DX,
        start.clone(),
        R_TOLERANCE,
        A_TOLERANCE,
    );

    let stats = dop.integrate().map_err(|_err| TransientError::Msg("integration error".to_string()))?;

    println!("x {:?}", dop.x_out());
    println!("y {:?}", dop.y_out());


    return Ok(TransientSolution {
        stats: stats,
        t: dop.x_out().clone(),
        out: dop.y_out().clone(),
    });
}


use nalgebra::DVector;
use nalgebra_sparse::{CsrMatrix, ops::{Op, serial::spmm_csr_dense}};
use num_traits::Float;

use crate::{graph::{Graph, NodeIndex, Edge}, power_system::{PsEdge, PsNode, NodeType}, traits::C32, matrix_builder::{CsrMatBuilder, MatBuilder}, utils::is_zero};

use super::SteadyStateError;

static TOLERANCE: f32 = 0.0001;
static DETECT_DIVERGENCE: f32 = 10.0;
static MAX_ITER: u32 = 50000;

#[derive(Debug)]
pub struct SteadyStateSolve{
    pub v: DVector<C32>,
    pub iter_count: u32,
}

pub fn steady_state_solve(graph: &Graph<PsNode, PsEdge>) -> Result<SteadyStateSolve, SteadyStateError>{
    let node_count = graph.get_node_count();
    let pq: DVector<C32> = DVector::<C32>::from_fn(node_count, |r, _c| {graph.get_node(NodeIndex(r)).data.gen - graph.get_node(NodeIndex(r)).data.load});
    let slack_node = graph.node_data.iter().find(|nd| nd.n_type == NodeType::Sk).unwrap();
    // let pv_nodes = graph.node_data.iter().enumerate().filter(|nd| nd.1.n_type == NodeType::PV).map(|n| n.0).collect::<Vec<usize>>();
    let (mat_y, diag_y) = create_adm_mat(node_count, graph);

    let diag_inv_y = diag_y.map(|y| y.inv());

    let mut curr_v: DVector<C32> = DVector::<C32>::from_fn(node_count, |_r, _c| { C32::new(1.0, 0.0) });

	let mut iter = 1;

	loop {
        if iter > MAX_ITER {
            return Err(SteadyStateError::NonConvergence); 
        }

        //TODO update pq
        let new_v = new_voltage(&curr_v, &pq, &mat_y, &diag_inv_y, node_count, slack_node.index.0);
        let manhattan_max = new_v.iter().map(|c| c.l1_norm()).max_by(|a,b| a.partial_cmp(b).unwrap()).unwrap();

        if manhattan_max > DETECT_DIVERGENCE {
            return Err(SteadyStateError::Divergence);
        }

        let norm = find_diff_norm(&curr_v, &new_v);

        if norm > TOLERANCE {
            return Ok(SteadyStateSolve{
                v: new_v,
                iter_count: iter,
            });
        }

        curr_v = new_v;
            iter += 1;
		}

	}

fn find_diff_norm(vec1: &DVector<C32>, vec2: &DVector<C32>) -> f32 {
    (vec1 - vec2).norm()
}

fn new_voltage(v: &DVector<C32>, pq: &DVector<C32>, mat_y: &CsrMatrix<C32>, diag: &DVector<C32>, node_count: usize, slack_node_index: usize) -> DVector<C32> {

    let mut temp_vec: DVector<C32> = DVector::<C32>::from_fn(node_count, |_r, _c| { C32::new(1.0, 0.0) });
    spmm_csr_dense(C32::new(0.0, 0.0), &mut temp_vec, C32::new(1.0, 0.0), Op::NoOp(mat_y), Op::NoOp(v));

    let mut new_v: DVector<C32> = DVector::<C32>::from_fn(node_count, |r, _c| { 
        let mut res = -temp_vec[r];

        if !is_zero(&pq[r]) {
            res += pq[r].conj() / v[r];
        }

        res *= diag[r];
        res
    });

    new_v[slack_node_index] = C32::new(1.0, 0.0);

	return new_v;
}

fn create_adm_mat(node_count: usize, graph: &Graph<PsNode, PsEdge>) -> ( CsrMatrix<C32>, DVector<C32> ) {
    let mut mut_diag_y: DVector<C32> = DVector::<C32>::from_fn(node_count, |_r, _c| {C32::new(0.0, 0.0)});
    let mut mut_mat_y = CsrMatBuilder::<C32>::new(node_count, node_count);

    graph.edges().iter().for_each(|e| {
        add_edge(&mut mut_mat_y, &mut mut_diag_y, e)
    });

    // println!("mut_diag_y {:#?}", mut_diag_y;

    (mut_mat_y.build().unwrap(), mut_diag_y)
}

fn add_edge(mut_mat_y: &mut CsrMatBuilder<C32>, mut_diag_y: &mut DVector<C32>, edge: &Edge<'_, PsEdge>) {
    let adm = edge.data.admittance();
    let half_line_charge = C32::new( 0.0, edge.data.line_charge() * 0.5);
    let n = edge.info.fnode.0;
    let m = edge.info.tnode.0;

    mut_mat_y.add(n, m, -adm);
    mut_mat_y.add(m, n, -adm);
    mut_diag_y[n] += adm + half_line_charge;
    mut_diag_y[m] += adm + half_line_charge;
}

mod tests {
    use std::{clone, convert::identity};

    use nalgebra_sparse::SparseEntry::Zero;

    use super::*;
    use crate::power_system::{*, self};

    const SIMPLE_STEADY_STATE_FILE_PATH: &str = "./grids/SimpleSteadyState/";
    const SIMPLE_STEADY_STATE_2_FILE_PATH: &str = "./grids/SimpleSteadyState2/";

    #[test]
    fn find_diff_length_test_zero() {
        let vec1 = DVector::from_vec(vec![C32::new(1.5,1.0), C32::new(0.0,1.0)]);
        let vec2 = DVector::from_vec(vec![C32::new(1.5,1.0), C32::new(0.0,1.0)]);
        let n = super::find_diff_norm(&vec1, &vec2);

        assert_eq!(n, 0.0);
    }

    #[test]
    fn find_diff_length_test() {
        let vec1 = DVector::from_vec(vec![C32::new(1.5,1.0), C32::new(0.0,1.0)]);
        let vec2 = DVector::from_vec(vec![C32::new(1.0,1.5), C32::new(0.0,1.0)]);
        let n = super::find_diff_norm(&vec1, &vec2);

        assert_eq!(n, 0.70710677);
    }

    #[test]
    fn create_adm_mat_test(){
        let ps = PowerSystem::from_files(SIMPLE_STEADY_STATE_FILE_PATH);

        let (adm_mat, diag) = super::create_adm_mat(ps.node_count(), &ps.g);

        let mat = C32::new(-6.289308, 22.012579);
        let diag_val = C32::new(6.289308, -22.00808);

        assert_eq!(adm_mat.get_row(0).unwrap().get_entry(1).unwrap().into_value(), mat);
        assert_eq!(adm_mat.get_row(1).unwrap().get_entry(0).unwrap().into_value(), mat);
        assert_eq!(adm_mat.get_row(0).unwrap().get_entry(0), Some(Zero));
        assert_eq!(adm_mat.get_row(1).unwrap().get_entry(1), Some(Zero));

        assert_eq!(diag.get(0).unwrap(), &diag_val);
        assert_eq!(diag.get(1).unwrap(), &diag_val);
    }

    #[test]
    fn steady_state_test(){
        let ps = PowerSystem::from_files(SIMPLE_STEADY_STATE_2_FILE_PATH);

        let res = super::steady_state_solve(&ps.g);

        let vec = res.unwrap().v;
        assert_eq!(vec.get(0).unwrap(), &C32::new(1.0,0.0));
        assert_eq!(vec.get(1).unwrap(), &C32::new(1.0156636,0.025887817));
        assert_eq!(vec.get(2).unwrap(), &C32::new(0.97542495,-0.020732194));
    }
}
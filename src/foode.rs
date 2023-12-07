
use std::{time::Duration, ops::Mul, usize};

use nalgebra::{OVector, Const, Dim, DefaultAllocator, allocator::Allocator, DimName, Dyn, DVector};
use nalgebra_sparse::csr::{CsrMatrix, CsrRow};
use nalgebra_sparse::ops::Op;
use nalgebra_sparse::ops::serial::spmm_csr_dense;
use ode_solvers::dop_shared::{System, Stats};

pub type State = ode_solvers::OVector<f64, Dyn>;
pub type Time = f64;

pub struct RowEle {
    row: usize,
    ele: f64,
}


#[derive(Debug, Clone)]
pub struct Foode {
    pub mat: CsrMatrix<f32>,
    pub stats: Stats,
    pub time: Duration
}


//     pub fn f(&self, t: f32, x: &State, dx: &mut State){

//     }

// impl System<State> for Foode {

//     fn system(&self, x: f64, y: &State, dy: &mut State){
//         (self.forcing_fn)(t, dx);
//         spmm_csr_dense(0.0,  dx, 1.0, Op::NoOp(&self.mat), Op::NoOp(x));
//     }

//     fn solout(&mut self, _x: f64, _y: &State, _dy: &State) -> bool { 
//         true
//      }
// }


#[cfg(test)]
mod tests {

    use std::time::Duration;
    use crate::{matrix_builder::MatBuilder, foode::State};
    use nalgebra::{OVector, Dyn, Vector1, DVector};
    use ode_solvers::{Dopri5, dop_shared::Stats};

    use crate::{runge_kutta::Rk4, foode::Foode, matrix_builder::CsrMatBuilder};

    fn t_squared(t: f32, _x: &f32) -> f32 {
        t * t
    }

    fn approx_equals(v1: f32, v2: f32, tolerance: f32) -> bool {
        (v1 - v2).abs() < tolerance
    }

    fn func(t: f32, x: &f32) -> f32 {
        x - t * t + 1.0
    }

    fn func_integrated(t: f32) -> f32 {
        -0.5 * (-2.0 * t * t - 4.0 * t + f32::exp(t) - 2.0)
    }

    fn func_vec(t: f32, dx: &mut DVector::<f32>) {
        dx[0] = t * t + 1.0;
    }

    // #[test]
    // fn test_integrate_rk4_ode_solver() {


    //     let system = Foode{
    //         mat: CsrMatBuilder::<f32>::new(1, 1).add(0, 0, 1.0).build().unwrap(),
    //         forcing_fn:  func_vec,
    //         stats: Stats {
    //             num_eval: 0,
    //             accepted_steps: 0,
    //             rejected_steps: 0,
    //         },
    //         time: Duration::from_secs(5)
    //     };

    //     // (f: F, x: f64, x_end: f64, dx: f64, y: OVector<T, D>, rtol: f64, atol: f64)


    //     let dvec = State::repeat(1, 0.5);

    //     let mut stepper: Dopri5<State, Foode> = Dopri5::new(system, 0.0, 1.0, 0.1, dvec, 0.1, 0.0001);

    //     let res = stepper.integrate();


    //     let y_out = stepper.y_out();

    //     println!("{:?}", y_out);
    // }
}

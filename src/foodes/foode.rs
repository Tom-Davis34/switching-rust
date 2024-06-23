use std::{ops::Mul, time::Duration, usize};

use super::*;
use nalgebra::{
    allocator::Allocator, Const, DVector, DefaultAllocator, Dim, DimName, Dyn, OVector,
};
use nalgebra_sparse::csr::{CsrMatrix, CsrRow};
use nalgebra_sparse::ops::serial::spmm_csr_dense;
use nalgebra_sparse::ops::Op;
use num_traits::Float;

const PI:f32 = 3.14159265359;

pub type State = DVector<f32>;
pub type Time = f32;


#[derive(Debug, Clone)]

pub struct TransientSolve {
    pub gen_curr_index: usize,
    pub a: CsrMatrix<f32>,
    pub a_tilder: CsrMatrix<f32>,
    pub b: Vec<f32>,
    pub switch_time: f32,
}

impl System<State> for TransientSolve {
    fn system(&self, x: f32, y: &State, dy: &mut State) {
        let forcing_term = ((2.0 * PI)*x*50.0).sin();
        
        let gen_curr = self.b.iter().map(|bv| bv*forcing_term).collect::<Vec<f32>>();

        for index in 0..gen_curr.len() {
            dy[index + self.gen_curr_index] += gen_curr[index];
        } 
        if x < self.switch_time {
            spmm_csr_dense(0.0, dy, 1.0, Op::NoOp(&self.a), Op::NoOp(y));
        } else {
            spmm_csr_dense(0.0, dy, 1.0, Op::NoOp(&self.a_tilder), Op::NoOp(y));
        }
    }
}

#[derive(Debug, Clone)]
pub struct Foode {
    pub mat: CsrMatrix<f32>,
    forcing_fn: fn(f32, &mut State),
}

impl System<State> for Foode {
    fn system(&self, x: f32, y: &State, dy: &mut State) {
        (self.forcing_fn)(x, dy);
        spmm_csr_dense(0.0, dy, 1.0, Op::NoOp(&self.mat), Op::NoOp(y));
    }
}

#[cfg(test)]
mod tests {
    use super::dop853::Dop853;
    use super::{Integratable, System};
    use super::dopri5::Dopri5;
    use super::rk4::Rk4;
    use super::{Stats, dopri5};
    use crate::{foodes::foode::State, matrix_builder::MatBuilder};
    use nalgebra::{DVector, Dim, Dyn, OVector, Vector1};
    use nalgebra_sparse::CsrMatrix;
    use std::iter::zip;
    use std::time::Duration;

    use crate::{foodes::foode::Foode, matrix_builder::CsrMatBuilder};

    macro_rules! assert_delta {
        ($x:expr, $y:expr, $d:expr) => {
            if !($x - $y < $d || $y - $x < $d) {
                panic!();
            }
        };
    }

    struct Solvers {
        rk4: Rk4<State, Foode>,
        dopri5: Dopri5<State, Foode>,
        dop853: Dop853<State, Foode>,
    }

    impl Solvers {
        pub fn new(
            x_start: f32,
            x_end: f32,
            delta_x: f32,
            y: State,
            tolerance: f32,
            mat: CsrMatrix<f32>,
            forcing_fn: fn(f32, &mut State),
        ) -> Solvers {
            let system = Foode {
                mat: mat,
                forcing_fn: forcing_fn,
            };

            Solvers {
                rk4: Rk4::new(system.clone(), x_start, x_end, delta_x, y.clone()),
                dopri5: Dopri5::new(
                    system.clone(),
                    x_start,
                    x_end,
                    delta_x,
                    y.clone(),
                    tolerance,
                    tolerance,
                ),
                dop853: Dop853::new(
                    system.clone(),
                    x_start,
                    x_end,
                    delta_x,
                    y.clone(),
                    tolerance,
                    tolerance,
                ),
            }
        }

        fn integrate(&mut self) {
            // let stats_rk4 = self.rk4.integrate();
            // println!("{:#?}", stats_rk4);

            let stats_dopri5 = self.dopri5.integrate();
            println!("{}", stats_dopri5.unwrap());

            let stats_dop853 = self.dop853.integrate();
            println!("{}", stats_dop853.unwrap());
        }

        fn check_self(&self, tolerance: f32, opt_f: Option<fn(f32) -> State>) {

            if opt_f.is_some() {
                let f = opt_f.unwrap();
                zip(self.dopri5.x_out(), self.dopri5.y_out()).for_each(|(x, y)| {
                    assert_delta_vec(&(f)(*x), y, tolerance);
                });
    
                zip(self.dop853.x_out(), self.dop853.y_out()).for_each(|(x, y)| {
                    assert_delta_vec(&(f)(*x), y, tolerance);
                });
            }

            zip(
                zip(self.dopri5.x_out(), self.dopri5.y_out()),
                zip(self.dop853.x_out(), self.dop853.y_out()),
            )
            .for_each(|((x1, y1), (x2, y2))| {
                assert_eq!(x1, x2);
                approx_equals(y1[0], y2[0], tolerance);
            });
        }
    }

    fn t_squared(t: f32, _x: &f32) -> f32 {
        t * t
    }

    fn approx_equals(v1: f32, v2: f32, tolerance: f32) -> bool {
        (v1 - v2).abs() < tolerance
    }

    fn assert_delta_vec(v1: &State, v2: &State, tolerance: f32) {
        zip(v1, v2).for_each(|v| assert_delta!(v.0, v.1, tolerance));
    }

    fn func(t: f32, x: &f32) -> f32 {
        x - t * t + 1.0
    }

    fn func_integrated(t: f32) -> State {
        State::repeat(1, -0.5 * (-2.0 * t * t - 4.0 * t + f32::exp(t) - 2.0))
    }

    fn t_squared_plus_one(t: f32, dx: &mut DVector<f32>) {
        dx[0] = t * t + 1.0;
    }

    #[test]
    fn test_integrate_1() {
        let tolerance = 1.0E-6;
        let dvec = State::repeat(1, 0.5);
        let mut mat_builder = CsrMatBuilder::<f32>::new(1, 1);
        mat_builder.add(0, 0, 1.0);
        let mat = mat_builder.build().unwrap();

        let mut solvers = Solvers::new(0.0, 1.0, 0.1, dvec, tolerance, mat, t_squared_plus_one);

        solvers.integrate();

        solvers.check_self(tolerance, Some(func_integrated));
    }

    fn forcing_fn_2(x: f32, dy: &mut DVector<f32>) {
        dy[0] = 12.0*f32::exp(x);
        dy[1] = 18.0*f32::exp(x);
    }

    #[test]
    fn test_integrate_2() {
        let tolerance = 1.0E-6;
        let dvec = State::repeat(2, 0.5);
        let mut mat_builder = CsrMatBuilder::<f32>::new(2, 2);
        mat_builder.add(0, 0, 1.0);
        mat_builder.add(0, 1, 2.0);
        mat_builder.add(1, 0, 4.0);
        mat_builder.add(1, 1, 3.0);
        let mat = mat_builder.build().unwrap();

        let mut solvers = Solvers::new(0.0, 1.0, 0.1, dvec, tolerance, mat, forcing_fn_2);

        solvers.integrate();

        solvers.check_self(tolerance, None);
    }

}

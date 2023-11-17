use nalgebra::{allocator::Allocator, DefaultAllocator, Dim, OVector, Scalar};
use num_traits::Zero;
// use simba::scalar::{ClosedAdd, ClosedMul, ClosedNeg, ClosedSub, SubsetOf};

pub struct Stats {
    pub num_eval: u32,
    pub accepted_steps: u32,
    pub rejected_steps: u32,
}

/// Structure containing the parameters for the numerical integration.
pub struct Rk4<X>
{
    t_start: f32,
    t_end: f32,
    step_size: f32,
    step_num: usize,
    t_out: Vec<f32>,
    x_out: Vec<X>,
    stats: Stats,
}



impl <X> Rk4<X>{

    pub fn new(t_start: f32, t_end: f32, step_size: f32) -> Self{
        let num = ((self.t_end - self.t_start) / self.step_size).ceil() as usize;

        return Rk4{
            t_start: t_start,
            t_end: t_end,
            step_size: step_size,
            step_num: num,
            t_out: Vec::with_capacity(num),
            x_out: Vec<X>::with_capacity(num),
            stats: Stats { num_eval: 0, accepted_steps: 0, rejected_steps: 0 }
        };
    }

    pub fn one_step_rk4_fn(start: &X, step_size: f32, f: fn(&X) -> X) -> X {
        let f1 = f(&start);
    
        let f2 = f(&(f1 * (step_size * 0.5)) + *start);
        let f3 = f(f2 * (step_size * 0.5) + start);
        let f4 = f(f3 * step_size + start);
    
        return start + (step_size / 6) * (f1 + 2 * f2 + 2 * f3 + f4);
    }

    pub fn run_fn(self: Self, start: &X, f: fn(&X) -> &X) -> Result<Stats, String> {

        let mut current = start; 

        for t in 0..self.step_num {
            let res =self.one_step_rk4_fn(current, self.step_size, f);

            self.x_out.push(res);
            self.t_out.push(t);
            self.stats.num_eval += 4;
            self.stats.accepted_steps += 1;

            current = &res;
        }

        return Result::OK(self.stats);
    }   


}




impl<T, D: Dim, F> Rk4<OVector<T, D>, F>
where
    f64: From<T>,
    T: Copy + SubsetOf<f64> + Scalar + ClosedAdd + ClosedMul + ClosedSub + ClosedNeg + Zero,
    F: System<OVector<T, D>>,
    OVector<T, D>: std::ops::Mul<f64, Output = OVector<T, D>>,
    DefaultAllocator: Allocator<T, D>,
{
    /// Default initializer for the structure
    ///
    /// # Arguments
    ///
    /// * `f`           - Structure implementing the System<V> trait
    /// * `x`           - Initial value of the independent variable (usually time)
    /// * `y`           - Initial value of the dependent variable(s)
    /// * `x_end`       - Final value of the independent variable
    /// * `step_size`   - Step size used in the method
    ///
    pub fn new(f: F, x: f64, y: OVector<T, D>, x_end: f64, step_size: f64) -> Self {
        Rk4 {
            f,
            x,
            y,
            x_end,
            step_size,
            half_step: step_size / 2.,
            x_out: Vec::new(),
            y_out: Vec::new(),
            stats: Stats::new(),
        }
    }

    /// Core integration method.
    pub fn integrate(&mut self) -> Result<Stats, IntegrationError> {
        // Save initial values
        self.x_out.push(self.x);
        self.y_out.push(self.y.clone());

        let num_steps = ((self.x_end - self.x) / self.step_size).ceil() as usize;
        for _ in 0..num_steps {
            let (x_new, y_new) = self.step();

            self.x_out.push(x_new);
            self.y_out.push(y_new.clone());

            self.x = x_new;
            self.y = y_new;

            self.stats.num_eval += 4;
            self.stats.accepted_steps += 1;
        }
        Ok(self.stats)
    }

    /// Performs one step of the Runge-Kutta 4 method.
    fn step(&self) -> (f64, OVector<T, D>) {
        let (rows, cols) = self.y.shape_generic();
        let mut k = vec![OVector::zeros_generic(rows, cols); 12];

        self.f.system(self.x, &self.y, &mut k[0]);
        self.f.system(
            self.x + self.half_step,
            &(self.y.clone() + k[0].clone() * self.half_step),
            &mut k[1],
        );
        self.f.system(
            self.x + self.half_step,
            &(self.y.clone() + k[1].clone() * self.half_step),
            &mut k[2],
        );
        self.f.system(
            self.x + self.step_size,
            &(self.y.clone() + k[2].clone() * self.step_size),
            &mut k[3],
        );

        let x_new = self.x + self.step_size;
        let y_new = &self.y
            + (k[0].clone() + k[1].clone() * 2.0 + k[2].clone() * 2.0 + k[3].clone())
                * (self.step_size / 6.0);
        (x_new, y_new)
    }

    /// Getter for the independent variable's output.
    pub fn x_out(&self) -> &Vec<f64> {
        &self.x_out
    }

    /// Getter for the dependent variables' output.
    pub fn y_out(&self) -> &Vec<OVector<T, D>> {
        &self.y_out
    }
}

#[cfg(test)]
mod tests {
    use crate::rk4::Rk4;
    use crate::{DVector, OVector, System, Vector1};
    use nalgebra::{allocator::Allocator, DefaultAllocator, Dim};

    struct Test1 {}
    impl<D: Dim> System<OVector<f64, D>> for Test1
    where
        DefaultAllocator: Allocator<f64, D>,
    {
        fn system(&self, x: f64, y: &OVector<f64, D>, dy: &mut OVector<f64, D>) {
            dy[0] = (x - y[0]) / 2.;
        }
    }

    struct Test2 {}
    impl<D: Dim> System<OVector<f64, D>> for Test2
    where
        DefaultAllocator: Allocator<f64, D>,
    {
        fn system(&self, x: f64, y: &OVector<f64, D>, dy: &mut OVector<f64, D>) {
            dy[0] = -2. * x - y[0];
        }
    }

    struct Test3 {}
    impl<D: Dim> System<OVector<f64, D>> for Test3
    where
        DefaultAllocator: Allocator<f64, D>,
    {
        fn system(&self, x: f64, y: &OVector<f64, D>, dy: &mut OVector<f64, D>) {
            dy[0] = (5. * x * x - y[0]) / (x + y[0]).exp();
        }
    }

    #[test]
    fn test_integrate_test1_svector() {
        let system = Test1 {};
        let mut stepper = Rk4::new(system, 0., Vector1::new(1.), 0.2, 0.1);
        let _ = stepper.integrate();
        let x_out = stepper.x_out();
        let y_out = stepper.y_out();
        assert!((*x_out.last().unwrap() - 0.2).abs() < 1.0E-8);
        assert!((&y_out[1][0] - 0.95369).abs() < 1.0E-5);
        assert!((&y_out[2][0] - 0.91451).abs() < 1.0E-5);
    }

    #[test]
    fn test_integrate_test2_svector() {
        let system = Test2 {};
        let mut stepper = Rk4::new(system, 0., Vector1::new(-1.), 0.5, 0.1);
        let _ = stepper.integrate();
        let x_out = stepper.x_out();
        let y_out = stepper.y_out();
        assert!((*x_out.last().unwrap() - 0.5).abs() < 1.0E-8);
        assert!((&y_out[3][0] + 0.82246).abs() < 1.0E-5);
        assert!((&y_out[5][0] + 0.81959).abs() < 1.0E-5);
    }

    #[test]
    fn test_integrate_test3_svector() {
        let system = Test3 {};
        let mut stepper = Rk4::new(system, 0., Vector1::new(1.), 1., 0.1);
        let _ = stepper.integrate();
        let out = stepper.y_out();
        assert!((&out[5][0] - 0.913059839).abs() < 1.0E-9);
        assert!((&out[8][0] - 0.9838057659).abs() < 1.0E-9);
        assert!((&out[10][0] - 1.0715783953).abs() < 1.0E-9);
    }

    #[test]
    fn test_integrate_test1_dvector() {
        let system = Test1 {};
        let mut stepper = Rk4::new(system, 0., DVector::from(vec![1.]), 0.2, 0.1);
        let _ = stepper.integrate();
        let x_out = stepper.x_out();
        let y_out = stepper.y_out();
        assert!((*x_out.last().unwrap() - 0.2).abs() < 1.0E-8);
        assert!((&y_out[1][0] - 0.95369).abs() < 1.0E-5);
        assert!((&y_out[2][0] - 0.91451).abs() < 1.0E-5);
    }

    #[test]
    fn test_integrate_test2_dvector() {
        let system = Test2 {};
        let mut stepper = Rk4::new(system, 0., DVector::from(vec![-1.]), 0.5, 0.1);
        let _ = stepper.integrate();
        let x_out = stepper.x_out();
        let y_out = stepper.y_out();
        assert!((*x_out.last().unwrap() - 0.5).abs() < 1.0E-8);
        assert!((&y_out[3][0] + 0.82246).abs() < 1.0E-5);
        assert!((&y_out[5][0] + 0.81959).abs() < 1.0E-5);
    }

    #[test]
    fn test_integrate_test3_dvector() {
        let system = Test3 {};
        let mut stepper = Rk4::new(system, 0., DVector::from(vec![1.]), 1., 0.1);
        let _ = stepper.integrate();
        let out = stepper.y_out();
        assert!((&out[5][0] - 0.913059839).abs() < 1.0E-9);
        assert!((&out[8][0] - 0.9838057659).abs() < 1.0E-9);
        assert!((&out[10][0] - 1.0715783953).abs() < 1.0E-9);
    }
}





use num_traits::{real::Real, Zero};
use std::{
    cmp::PartialOrd,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign}
};


pub type Float = f32;

pub trait Integrable: Copy + Zero + PartialEq 
where
    Self: Add<Output = Self>,
    Self: Sub<Output = Self>,
    Self: Mul<Float, Output = Self>,
    Self: Div<Float, Output = Self>,
    Self: Neg<Output = Self>, 
{}


fn runge_kutta4_step<T>(&start: &T, dt: Float, f: fn(&T) -> T) -> T 
where T: Integrable{
    
    let v1 = f(&start);
	let v2 = f(&(v1 * (dt / 2.0) + start));
	let v3 = f(&(v2 * (dt / 2.0) + start));
	let v4 = f(&(v3 * dt + start));

	return start + (v1 + (v2 * 2.0) + (v3 * 2.0) + v4) * (dt / 6.0);
}

pub fn runge_kutta4<T>(&start: &T,num: usize, dt: Float, f: fn(&T) -> T) -> Vec<T> 
where T: Integrable{
    
    let mut ret_val = Vec::<T>::with_capacity(num + 1);
    
    let mut current_value = start;

    (0..num).for_each(|n| {
        ret_val.push(current_value);

        current_value = runge_kutta4_step(&current_value, dt, f);
    });

    return ret_val;
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn blahTest() {
        blah()
    }

}


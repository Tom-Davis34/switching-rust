use nalgebra::{allocator::Allocator, DefaultAllocator, Dim, OVector, Scalar};
use num_traits::Zero;

use crate::traits::VectorSpace;
// use simba::scalar::{ClosedAdd, ClosedMul, ClosedNeg, ClosedSub, SubsetOf};

#[derive(Debug, Clone, Copy)]
pub struct Stats {
    pub num_eval: u32,
    pub accepted_steps: u32,
    pub rejected_steps: u32,
}


#[derive(Debug, Clone, Copy)]
pub struct FunSample<X> where X: VectorSpace{
    t: f32,
    x: X
}

/// Structure containing the parameters for the numerical integration.
#[derive(Debug, Clone)]
pub struct Rk4<X> where X: VectorSpace
{
    t_start: f32,
    t_end: f32,
    step_size: f32,
    step_num: usize,
    fun: Vec<FunSample<X>>,
    stats: Stats,
}



impl <X> Rk4<X> where X: VectorSpace{

    pub fn new(t_start: f32, t_end: f32, step_size: f32) -> Self{
        let num = ((t_end - t_start) / step_size).ceil() as usize;

        return Rk4{
            t_start: t_start,
            t_end: t_end,
            step_size: step_size,
            step_num: num,
            fun: Vec::with_capacity(num),
            stats: Stats { num_eval: 0, accepted_steps: 0, rejected_steps: 0 }
        };
    }

    pub fn one_step_rk4_fn(x0: &X, t0: f32, h: &f32, f: fn(f32, &X) -> X) -> X {
        let half = h / 2.0;
        
        let k1 = f(t0, x0);
        let k2 = f(t0 + half, &(k1 * half + *x0));
        let k3 = f(t0 + half, &(k2 * half + *x0));
        let k4 = f(t0 + h, &(k3 * *h + *x0));
    
        return *x0 + (k1 * 1.0/6.0 +  k2 * 2.0 /6.0 + k3*2.0 / 6.0 + k4 * 1.0/6.0) * (*h);
    }

    pub fn run_fn(self: &mut Self, start: &X, f: fn(f32, &X) -> X) -> Result<Stats, String> {

        let mut current = start.to_owned(); 
        let mut t = self.t_start; 

        self.fun.push(FunSample { t: t, x: current });

        for n in 0..self.step_num {
            t = self.step_size*n as f32;

            current = Rk4::<X>::one_step_rk4_fn(&current, t, &self.step_size, f);

            self.fun.push(FunSample { t: t + self.step_size, x: current });
            self.stats.num_eval += 4;
            self.stats.accepted_steps += 1;
        }

        return Result::Ok(self.stats);
    }   


}

#[cfg(test)]
mod tests {

    use crate::runge_kutta::Rk4;

    fn t_squared(t:f32, _x:&f32) -> f32{
        t * t
    }

    fn approx_equals(v1: f32, v2: f32, tolerance: f32) -> bool{
        (v1 - v2).abs() < tolerance
    }

    #[test]
    fn test_integrate_rk4_simple() {
        let mut rk4: Rk4<f32> = Rk4::<f32>::new(0.0, 1.0, 0.001);

        let res = rk4.run_fn(&0.0, t_squared);

        assert!(res.is_ok());
        rk4.fun.iter().for_each(|sample| {
            println!("{:?}", sample);
            assert!(approx_equals(sample.x, sample.t * sample.t, 0.001));
        });
    }

    fn func(t:f32, x:&f32) -> f32{
        x - t*t + 1.0
    }

    fn func_integrated(t:f32) -> f32{
        -0.5 * (-2.0*t*t - 4.0*t + f32::exp(t) - 2.0)
    }

    #[test]
    fn test_integrate_rk4() {
        let mut rk4: Rk4<f32> = Rk4::<f32>::new(0.0, 1.0, 0.1);

        let res = rk4.run_fn(&0.5, func);

        assert!(res.is_ok());
        rk4.fun.iter().for_each(|sample| {
            println!("{:?}", sample);
            assert!(approx_equals(sample.x, func_integrated(sample.t), 0.001));
        });
    }
}
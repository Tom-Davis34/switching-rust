use nalgebra::Scalar;
use num_complex::{Complex64, Complex32, Complex};
use num_traits::Zero;
use std::{
    cmp::PartialOrd,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign}, fmt
};

pub type C32 = Complex<f32>;
pub type C64 = Complex<f64>;

pub trait VectorSpace: Copy + Zero + PartialEq + std::fmt::Debug + AddAssign + DivAssign + MulAssign + SubAssign + Scalar
where
    Self: Add<Output = Self>,
    Self: Sub<Output = Self>,
    Self: Mul<f32, Output = Self>,
    Self: Div<f32, Output = Self>,
    Self: Neg<Output = Self>, 

{}

trait Printable{
    fn to_string(&self) -> String;
}
impl Printable for C32 {
    fn to_string(&self) -> String{
        format!("{}, {}i", self.re, self.im)
    }
}

impl VectorSpace for f32 {
    
}

impl VectorSpace for C32 {
    
}


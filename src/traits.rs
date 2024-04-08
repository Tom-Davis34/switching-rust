use nalgebra::Scalar;
use num_complex::{Complex64, Complex32, Complex};
use num_traits::Zero;
use std::{
    cmp::PartialOrd,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign}, fmt
};

pub type C32 = Complex<f32>;
pub type C64 = Complex<f64>;



trait Printable{
    fn to_string(&self) -> String;
}
impl Printable for C32 {
    fn to_string(&self) -> String{
        format!("{}, {}i", self.re, self.im)
    }
}

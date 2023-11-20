use num_traits::Zero;
use std::{
    cmp::PartialOrd,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign}
};

pub trait VectorSpace: Copy + Zero + PartialEq + std::fmt::Debug + AddAssign + DivAssign + MulAssign + SubAssign
where
    Self: Add<Output = Self>,
    Self: Sub<Output = Self>,
    Self: Mul<f32, Output = Self>,
    Self: Div<f32, Output = Self>,
    Self: Neg<Output = Self>, 

{}

impl VectorSpace for f32 {
    
}
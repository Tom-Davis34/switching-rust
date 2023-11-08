use num_traits::{real::Real, Zero};
use std::{
    cmp::PartialOrd,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};



pub trait VectorSpace: Copy + Zero + PartialEq 
where
    Self: Add<Output = Self>,
    Self: Sub<Output = Self>,
    Self: Mul<Self::Scalar, Output = Self>,
    Self: Div<Self::Scalar, Output = Self>,
    Self: Neg<Output = Self>, 
{
    type Scalar: Real + PartialOrd;
}

impl VectorSpace for f32 {
    type Scalar = f32;
}
impl VectorSpace for f64 {
    type Scalar = f64;
}

/// This trait is automatically implemented for vector spaces, which also implement assignment operations.
pub trait VectorSpaceAssign:
    VectorSpace + AddAssign + SubAssign + MulAssign<Self::Scalar> + DivAssign<Self::Scalar>
{
}
impl<T> VectorSpaceAssign for T where
    T: VectorSpace + AddAssign + SubAssign + MulAssign<Self::Scalar> + DivAssign<Self::Scalar>
{
}

pub fn runge_kutta4<T>(&start: &T, dt: Real, f: fn(&T) -> &T) -> T where T: VectorSpace {
    let f1 = f(start);

    let blah = start * dt;

	let f2 = f(&(f1 * (dt * 0.5)) + *start);
	let f3 = f(f2 * (dt / 2) + start);
	let f4 = f(f3 * dt + start);

	return start + (dt / 6) * (f1 + 2 * f2 + 2 * f3 + f4);
}




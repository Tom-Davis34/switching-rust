#[macro_use]
extern crate approx; // For the macro relative_eq!
extern crate nalgebra as na;
use na::{Vector3, Rotation3};
extern crate nalgebra_sparse;
use nalgebra_sparse::{csr::CsrMatrix, csc::CscMatrix, coo::CooMatrix};



pub mod matrix;
pub mod runge_kutta;

fn main() {
    let axis  = Vector3::x_axis();
    let angle = 1.57;
    let b     = Rotation3::from_axis_angle(&axis, angle);

    let res = relative_eq!(b.axis().unwrap(), axis);
    let res2 = relative_eq!(b.angle(), angle);
    println!("{}", res);
}
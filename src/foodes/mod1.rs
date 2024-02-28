
mod butcher_tableau;
mod controller;
mod dop_shared;
mod dop853;
mod dopri5;
mod foode;
mod rk4;

    /// * `f`       - Structure implementing the System<V> trait
    /// * `x`       - Initial value of the independent variable (usually time)
    /// * `x_end`   - Final value of the independent variable
    /// * `dx`      - Increment in the dense output. This argument has no effect if the output type is Sparse
    /// * `y`       - Initial value of the dependent variable(s)
    /// * `rtol`    - Relative tolerance used in the computation of the adaptive step size
    /// * `atol`    - Absolute tolerance used in the computation of the adaptive step size
    
pub struct Dop853Config {
    x_end: f32,
    dx: f32,
    rtol: f32,
    atol: f32,
}    

pub struct Config {
    x_end: f32,
    step_size: f32,
    rtol: f32,
    atol: f32,
}  

enum IntegrationMethod {
    Dop853,
    Dopri5,
    Rk4(),
}

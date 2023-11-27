
use nalgebra::{OVector, Dyn};
use nalgebra_sparse::csr::{CsrMatrix, CsrRow};

pub struct RowEle {
    row: usize,
    ele: f32,
}


#[derive(Debug, Clone)]
pub struct Foode {
    pub mat: CsrMatrix<f32>,
    pub forcing_fn: fn (f32) -> OVector<f32, Dyn>
}

// y'(t)=f(t,y(t))

impl Foode {
    pub fn f(&self, t: f32, x: OVector<f32, Dyn>) -> OVector<f32, Dyn>{
        self.mat * x + (self.forcing_fn)(t)
    }
}
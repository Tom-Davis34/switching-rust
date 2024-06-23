
use nalgebra::Scalar;
use nalgebra_sparse::{csr::{CsrMatrix, CsrRow}};
use nalgebra::Complex;
use num_traits::Float;

use crate::traits::C32;

trait PrintDenseMat<T> where T: Sized {
    fn print_mat_as_dense(&self);
}

pub fn print_mat_as_dense(mat: &CsrMatrix<C32> ) {

    for row in mat.row_iter() {
        println!("{:#?}",row);
    }

}

impl<T> PrintDenseMat<T> for CsrMatrix<T> 
where T: Scalar{

    fn print_mat_as_dense(&self) {
        for row in self.row_iter() {
            println!("{:?}",row)
        }
    
    }
}

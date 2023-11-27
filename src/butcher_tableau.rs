use nalgebra_sparse::{CsrMatrix, SparseFormatError};

use crate::{matrix_builder::{CsrMatBuilder, MatBuilder}, traits::VectorSpace};

use once_cell::sync::Lazy;

static BT_RK4: Lazy<ButcherTableau> = Lazy::new(|| {
    return ButcherTableau::new(
        4,
        vec![0.5, 0.5, 1.0],
        vec![
            vec![0.5],
            vec![0.0, 0.5],
            vec![0.0, 0.0, 1.0]
        ],
        vec![1.0/6.0, 1.0/3.0, 1.0/3.0, 1.0/6.0],
        Option::None
    );
});

#[derive(Debug)]
pub struct ButcherTableau{
    pub num: usize,
    pub time_offsets: Vec<f32>,
    pub a_mat: CsrMatrix<f32>,
 
    pub  b: Vec<f32>,
    pub b_opt: Option<Vec<f32>>,
}

impl ButcherTableau{
    pub fn new(num: usize, time_offsets: Vec<f32>, mat: Vec<Vec<f32>>, b: Vec<f32>, b_opt: Option<Vec<f32>>) -> Self {
        if time_offsets.len() != num - 1 {
            panic!("time_offset.len() is not equal to 'num - 1'")
        }

        if b.len() != num {
            panic!("b.len() is not equal to 'num'")
        }
        if b.iter().sum::<f32>() != 1.0 {
            panic!("b should add up to one")
        }

        if b_opt.is_some() && b_opt.as_ref().unwrap().len() != num {
            panic!("b_opt.len() is not equal to 'num'")
        }
        if b_opt.is_some() && b_opt.as_ref().unwrap().iter().sum::<f32>() != 1.0  {
            panic!("b_opt should add up to one")
        }

        if mat.len() != num - 1 {
            panic!("mat.len() is not equal to 'num - 1'")
        }

        let mut mat_builder = CsrMatBuilder::<f32>::new(num, num); 

        let mut cur_row_index = 0;
        for row in mat{
            row.iter().enumerate().for_each( | (index, ele) | mat_builder.add_mut(cur_row_index, index, *ele));
        
            cur_row_index += 1;
        }

        let mat = mat_builder.build().unwrap();
        assert_eq!(mat.lower_triangle(), mat);

        Self {
            num: num,
            time_offsets: time_offsets,
            a_mat: mat,
            b: b,
            b_opt: b_opt
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::butcher_tableau::BT_RK4;

    #[test]
    fn test_rk4() {
        BT_RK4.b.len();

        println!("{:?}", BT_RK4);
    }

}
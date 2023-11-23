use nalgebra_sparse::{CsrMatrix, SparseFormatError};

use crate::{matrix_builder::{CsrMatBuilder, MatBuilder}, traits::VectorSpace};

pub struct ButcherTableau<T>
where
    T: VectorSpace,
{
    num: usize,
    time_offsets: Vec<f32>,
    a_mat: CsrMatrix<T>,

    b: Vec<f32>,
    b_opt: Option<Vec<f32>>,
}

static bt_rk4: ButcherTableau<f32> = build_rk4();
const fn build_rk4() -> ButcherTableau<f32>{
    return ButcherTableauBuilder::<f32>::new(
        4,
        vec![0.5, 0.5, 1.0],
        vec![1.0/6.0, 1.0/3.0, 1.0/3.0, 1.0/6.0],
        Option::None
    )
    .add(vec![0.5])
    .add(vec![0.0, 0.5])
    .add(vec![0.0, 0.0, 1.0])
    .build().unwrap();
}

pub struct ButcherTableauBuilder<T>
where
    T: VectorSpace,
{
    num: usize,
    cur_row_index: usize,
    time_offsets: Vec<f32>,
    a_mat: CsrMatBuilder<T>,

    b: Vec<f32>,
    b_opt: Option<Vec<f32>>,
}

impl<T> ButcherTableauBuilder<T> where
T: VectorSpace,
{

    pub fn new(num: usize, time_offsets: Vec<f32>, b: Vec<f32>, b_opt: Option<Vec<f32>>) -> Self {
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

        Self {
            num: num,
            cur_row_index: 0,
            time_offsets: time_offsets,
            a_mat: CsrMatBuilder::new(num, num),
            b: b,
            b_opt: b_opt
        }
    }

    fn add(mut self: Self, row: Vec<T>) -> Self{
        row.iter().enumerate().for_each( | (index, ele) | self.a_mat = self.a_mat.add(self.cur_row_index, index, *ele));

        self.cur_row_index += 1;
        
        return self;
    }

    fn build(self: Self) -> Result<ButcherTableau<T>, SparseFormatError>{
        let mat = self.a_mat.build()?;

        let bt = ButcherTableau::<T>{
            num: self.num,
            time_offsets: self.time_offsets,
            a_mat: mat,
            b: self.b,
            b_opt: self.b_opt,
        };

        return Ok(bt);
    }
}


#[cfg(test)]
mod tests {


    #[test]
    fn test_rk4() {

    }

}
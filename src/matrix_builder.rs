use nalgebra_sparse::{csr::CsrMatrix, CooMatrix};
use std::collections::HashMap;

use crate::traits::VectorSpace;

trait MatBuilder<T>
where
    T: VectorSpace,
{
    fn add(self: Self, row: usize, col: usize, ele: T) -> Self;
    fn build(self: Self) -> CsrMatrix<T>;
}

pub struct ColVal<T> {
    col: usize,
    ele: T,
}

pub struct RowMatBuilder<T>
where
    T: VectorSpace,
{
    row_num: usize,
    col_num: usize,
    rows: HashMap<usize, Vec<ColVal<T>>>,
}

impl<T> RowMatBuilder<T>
where
    T: VectorSpace,
{
    pub fn new() -> Self {
        Self {
            rows: HashMap::new(),
        }
    }
}

impl<T> MatBuilder<T> for RowMatBuilder<T>
where
    T: VectorSpace,
{
    fn add(mut self: Self, row: usize, col: usize, ele: T) -> Self {
        match self.rows.get(&row) {
            Some(vec) => {
                let opt_cell = vec.iter().find(|val| val.col == col);

                match opt_cell {
                    Some(cell) => {
                        cell.ele += ele;
                        return self;
                    }
                    None => {
                        vec.push(ColVal { col: col, ele: ele });
                        return self;
                    }
                }
            }
            None => {
                self.rows.insert(row, vec![ColVal { col: col, ele: ele }]);
                return self;
            }
        }
    }

    fn build(self: Self) -> CsrMatrix<T> {
        let mut coo = CooMatrix::<T>::new(self.row_num, self.col_num);

        self.rows
            .iter()
            .flat_map(|r| r.1.iter().map(|col_val| (r.0, col_val.col, col_val.ele)))
            .for_each(|triple| coo.push(*triple.0, triple.1, triple.2));

        return CsrMatrix::from(&coo);
    }
}

#[cfg(test)]
mod tests {

    use crate::matrix_builder::RowMatBuilder;

    #[test]
    fn test_coo_vs_mat_builder() {
        
    }
}

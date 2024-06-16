use nalgebra::Complex;
use nalgebra_sparse::{csr::CsrMatrix, CooMatrix, SparseFormatError};
use num_complex::{Complex32, Complex64};
use num_traits::Zero;
use std::{collections::HashMap, ops::AddAssign, panic};

use nalgebra::Scalar;

pub trait MatBuilder<T>
where
    T: Scalar + Zero + AddAssign,
{
    fn add(&mut self, row: usize, col: usize, ele: T);
    fn build(&self) -> Result<CsrMatrix<T>, SparseFormatError>;
}

pub struct ColEle<T> {
    col: usize,
    ele: T,
}

pub struct CsrMatBuilder<T>
where
    T: Scalar + Zero + AddAssign,
{
    row_num: usize,
    col_num: usize,
    rows: Vec<Vec<ColEle<T>>>,
}

pub struct Triple<T> {
    row: usize,
    col: usize,
    ele: T,
}

impl<T> CsrMatBuilder<T>
where
    T: Scalar + Zero + AddAssign,
{
    pub fn new(row_num: usize, col_num: usize) -> Self {
        let mut rows = Vec::with_capacity(row_num);

        for _i in 0..row_num {
            rows.push(Vec::new());
        }

        Self {
            row_num: row_num,
            col_num: col_num,
            rows: rows,
        }
    }
}

impl<T> MatBuilder<T> for CsrMatBuilder<T>
where
    T: Scalar + Zero + AddAssign,
{
    fn add(&mut self, row: usize, col: usize, ele: T) {
        if ele.is_zero() {
            return;
        }

        match self.rows.get_mut(row) {
            Some(vec) => {
                let opt_cell = vec.iter_mut().find(|val| val.col == col);

                match opt_cell {
                    Some(cell) => {
                        cell.ele += ele;
                        return;
                    }
                    None => {
                        vec.push(ColEle { col: col, ele });
                        return;
                    }
                }
            }
            None => {
                self.rows.insert(row, vec![ColEle { col: col, ele: ele }]);
                return;
            }
        }
    }

    fn build(&self) -> Result<CsrMatrix<T>, SparseFormatError> {
        let mut cursor = 0;

        let row_offset = self.rows.iter().fold(vec![0], |mut vec, cols_ele| {
            cursor += cols_ele.len();
            vec.push(cursor);
            return vec;
        });

        let col_indices = self
            .rows
            .iter()
            .flat_map(|row| row.iter())
            .map(|col_ele| col_ele.col)
            .collect();
        let values = self
            .rows
            .iter()
            .flat_map(|row| row.iter())
            .map(|col_ele| col_ele.ele.clone())
            .collect();

        // println!("row_offset {:?}", row_offset);
        // println!("col_indices {:?}", col_indices);
        // println!("values {:?}", values);

        return CsrMatrix::try_from_unsorted_csr_data(
            self.row_num,
            self.col_num,
            row_offset,
            col_indices,
            values,
        );
    }
}

#[cfg(test)]
mod tests {

    use nalgebra::{Matrix3x4, Scalar};
    use nalgebra_sparse::CsrMatrix;
    use num_complex::Complex;
    use num_traits::Zero;

    use crate::{matrix_builder::CsrMatBuilder, traits::C32};

    use super::MatBuilder;

    fn matrices_eq<T>(m1: CsrMatrix<T>, m2: CsrMatrix<T>) -> Result<(), &'static str>
    where
        T: std::cmp::PartialEq,
    {
        let data1 = m1.csr_data();
        let data2 = m2.csr_data();

        if data1.0 != data2.0 {
            return Err("Row offsets are not Equal");
        }
        if data1.1 != data2.1 {
            return Err("Column indices are not Equal");
        }
        if data1.2 != data2.2 {
            return Err("Elements are not Equal");
        }

        Ok(())
    }

    fn print_mat_as_dense<T>(mat: &CsrMatrix<T>)
    where
        T: Scalar + Zero + std::fmt::Display + std::fmt::LowerExp,
    {
        for row in mat.row_iter() {
            print!("[");
            (0..mat.ncols()).for_each(|col_index| {
                print!(
                    " {:.2}",
                    row.get_entry(col_index)
                        .map_or(T::zero(), |v| v.into_value())
                )
            });
            print!(" ]");
            println!();
        }
    }

    #[test]
    fn test_coo_vs_mat_builder() {
        let row_offsets = vec![0, 3, 3, 5];
        let col_indices = vec![0, 1, 3, 1, 2];
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        // The dense representation of the CSR data, for comparison
        let _dense = Matrix3x4::new(1.0, 2.0, 0.0, 3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 4.0, 5.0, 0.0);

        // The constructor validates the raw CSR data and returns an error if it is invalid.
        let expected_csr = CsrMatrix::try_from_csr_data(3, 4, row_offsets, col_indices, values)
            .expect("CSR data must conform to format specifications");

        let mut mat_builder = CsrMatBuilder::<f32>::new(3, 4);
        mat_builder.add(0, 0, 1.0);
        mat_builder.add(0, 1, 2.0);
        mat_builder.add(0, 3, 1.0);
        mat_builder.add(0, 3, 2.0);
        mat_builder.add(2, 1, 4.0);
        mat_builder.add(2, 2, 5.0);

        let actual_csr = mat_builder.build().unwrap();

        println!("expected_csr");
        print_mat_as_dense(&expected_csr);
        println!("actual_csr");
        print_mat_as_dense(&actual_csr);

        // println!("{:?}",expected_csr);
        // println!("{:?}",actual_csr);
        assert!(matrices_eq(expected_csr, actual_csr).is_ok());
    }

    #[test]
    fn test_complex_matrices() {
        let row_offsets = vec![0, 3, 3, 5];
        let col_indices = vec![0, 1, 3, 1, 2];
        let values: Vec<C32> = vec![
            C32::new(1.0, 0.0),
            C32::new(2.0, 0.0),
            C32::new(3.0, 0.0),
            C32::new(4.0, 0.0),
            C32::new(5.0, 0.0),
        ];

        // The dense representation of the CSR data, for comparison
        let _dense = Matrix3x4::new(1.0, 2.0, 0.0, 3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 4.0, 5.0, 0.0);

        // The constructor validates the raw CSR data and returns an error if it is invalid.
        let expected_csr = CsrMatrix::try_from_csr_data(3, 4, row_offsets, col_indices, values)
            .expect("CSR data must conform to format specifications");

        let mut mat_builder = CsrMatBuilder::<C32>::new(3, 4);
        mat_builder.add(0, 0, C32::new(1.0, 0.0));
        mat_builder.add(0, 1, C32::new(2.0, 0.0));
        mat_builder.add(0, 3, C32::new(1.0, 0.0));
        mat_builder.add(0, 3, C32::new(2.0, 0.0));
        mat_builder.add(2, 1, C32::new(4.0, 0.0));
        mat_builder.add(2, 2, C32::new(5.0, 0.0));

        let actual_csr = mat_builder.build().unwrap();

        println!("expected_csr");
        print_mat_as_dense(&expected_csr);
        println!("actual_csr");
        print_mat_as_dense(&actual_csr);

        // println!("{:?}",expected_csr);
        // println!("{:?}",actual_csr);
        assert!(matrices_eq(expected_csr, actual_csr).is_ok());
    }
}

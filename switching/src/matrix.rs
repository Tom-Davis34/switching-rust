use nalgebra_sparse::{csr::CsrMatrix};
use std::collections::HashMap;

trait MatBuilder<T> {
    fn add(&mut self, row: usize, col: usize, ele: T) -> Self;
}

pub struct ColVal<T> {
    col: usize, 
    ele: T
}

pub struct RowMatBuilder<T>{
    rows: HashMap<usize, Vec<ColVal<T>>>
}

impl <T> RowMatBuilder<T> {
    pub fn new()  -> Self {
        Self { 
            rows: HashMap::new()
         }
    }
}

impl <T> MatBuilder<T> for RowMatBuilder<T>{
    fn add(&mut self, row: usize, col: usize, ele: T) -> Self{
        match self.rows.get(&row) {
            Some(vec) => {
                todo!()
            }
            None => todo!(),
        }
    }
}

fn blah() {

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blahTest() {
        blah()
    }
}

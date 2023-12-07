use nalgebra::Const;

#[derive(Debug)]
pub struct ButcherTableau<const R: usize>  where [(); ((R - 1) * R ) / 2]: Sized {
    pub time_offsets: [f32; R],

    pub a_mat: [f32; ((R - 1) * R ) / 2],

    pub b: [f32; R],
    pub b_opt: Option<[f32; R]>,
}

impl<const R: usize> ButcherTableau<R> where [(); ((R - 1) * R ) / 2]: Sized {
    const R: usize = R;

    #[inline]
    pub fn dim(&self) -> usize {
        R
    }

    pub fn row(&self, row_index: usize) -> &[f32] {
        let r = (row_index * (row_index - 1)) / 2;
        return &self.a_mat[r..r + row_index];
    }
}

const RK4: ButcherTableau<4> = ButcherTableau {
    time_offsets: [0.0, 0.5, 0.5, 1.0],
    a_mat: [
        0.5, 
        0.0, 0.5, 
        0.0, 0.0, 1.0
        ],
    b: [1.0 / 6.0, 1.0 / 3.0, 1.0 / 3.0, 1.0 / 6.0],
    b_opt: None,
};

#[cfg(test)]
mod tests {

    use super::RK4;

    #[test]
    fn test_rk4() {
        unsafe{
            assert_eq!([0.0, 0.5, 0.5, 1.0], RK4.time_offsets);

            assert_eq!([0.5], RK4.row(1));
            assert_eq!([0.0, 0.5], RK4.row(2));
            assert_eq!([0.0, 0.0, 1.0], RK4.row(3));
            
            assert_eq!([1.0 / 6.0, 1.0 / 3.0, 1.0 / 3.0, 1.0 / 6.0], RK4.b);

            assert_eq!(4 as usize, RK4.dim());
        }
    }
}

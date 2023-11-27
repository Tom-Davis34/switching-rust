use nalgebra::Const;

#[derive(Debug)]
struct ButcherTableau<const R: usize> {
    time_offsets: [f32; R],

    a_mat: [[f32]; ((R - 1) * (R - 2)) / 2],

    b: [f32; R],
    b_opt: Option<[f32; R]>,
}

impl<const R: usize> ButcherTableau<R> {
    const fn size() {
        return R;
    }

    const fn a_mat_index_for_row(row_index: usize) -> usize {
        (row_index * (row_index - 1)) / 2
    }

    const unsafe fn row(row_index: usize) -> &[f32; row_index] {
        unsafe {
            return a_mat[a_mat_index_for_row(row_index)..a_mat_index_for_row(row_index + 1)];
        }
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
    use super;

    #[test]
    fn test_rk4() {
        assert_eq!([0.0, 0.5, 0.5, 1.0], RK4.time_offsets);

        assert_eq!([0.5], RK4.row(1));
        assert_eq!([0.0, 0.5], RK4.row(2));
        assert_eq!([0.0, 0.0, 0.1], RK4.row(3));
        
        assert_eq!(1.0 / 6.0, 1.0 / 3.0, 1.0 / 3.0, 1.0 / 6.0, RK4.b);
    }
}

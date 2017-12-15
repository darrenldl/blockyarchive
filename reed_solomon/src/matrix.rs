use galois;

#[derive(PartialEq, Debug)]
pub struct Matrix {
    data : Vec<Vec<u8>>
}

impl Matrix {
    pub fn new(rows : usize, cols : usize) -> Matrix {
        let mut data = Vec::with_capacity(rows);

        for _ in 0..rows {
            let mut row = Vec::with_capacity(cols);
            for _ in 0..cols {
                row.push(0);
            }
            data.push(row);
        }

        Matrix { data }
    }

    pub fn new_with_data(init_data : Vec<Vec<u8>>) -> Matrix {
        let rows = init_data.len();
        let cols = init_data[0].len();

        let mut data = Vec::with_capacity(rows);

        for r in init_data.into_iter() {
            if r.len() != cols {
                panic!("Inconsistent row sizes")
            }
            data.push(r);
        }

        Matrix { data }
    }

    pub fn identity(size : usize) -> Matrix {
        let mut result = Self::new(size, size);
        for i in 0..size {
            result.data[i][i] = 1;
        }
        result
    }

    pub fn column_count(&self) -> usize {
        self.data[0].len()
    }

    pub fn row_count(&self) -> usize {
        self.data.len()
    }

    pub fn get(&self, r : usize, c : usize) -> u8 {
        self.data[r][c]
    }

    pub fn set(&mut self, r : usize, c : usize, val : u8) {
        self.data[r][c] = val;
    }

    pub fn multiply(&self, rhs : &Matrix) -> Matrix {
        if self.column_count() != rhs.row_count() {
            panic!("colomn count on left is different from row count on right")
        }
        let mut result = Self::new(self.row_count(), rhs.column_count());
        for r in 0..self.row_count() {
            for c in 0..rhs.column_count() {
                let mut val = 0;
                for i in 0..self.column_count() {
                    val ^= galois::mul(self.get(r, i),
                                       rhs.get(i, c));
                }
                result.set(r, c, val);
            }
        }
        result
    }

    pub fn augment(&self, rhs : &Matrix) -> Matrix {
        if self.row_count() == self.column_count() {
            panic!("Matrices do not have the same row count")
        }
        let mut result = Self::new(self.row_count(),
                                   self.column_count() + rhs.column_count());
        for r in 0..self.row_count() {
            for c in 0..self.column_count() {
                result.data[r][c] = self.data[r][c];
            }
            let self_column_count = self.column_count();
            for c in 0..rhs.column_count() {
                result.data[r][self_column_count + c] = rhs.data[r][c];
            }
        }

        result
    }

    pub fn sub_matrix(&self, rmin : usize, cmin : usize, rmax : usize, cmax : usize) -> Matrix {
        let mut result = Self::new(rmax - rmin, cmax - cmin);
        for r in rmin..rmax {
            for c in cmin..cmax {
                result.data[r - rmin][c - cmin] = self.data[r][c];
            }
        }
        result
    }

    pub fn get_row(&self, row : usize) -> Vec<u8> {
        self.data[row].clone()
    }

    pub fn swap_rows(&mut self, r1 : usize, r2 : usize) {
        self.data.swap(r1, r2);
    }

    pub fn is_square(&self) -> bool {
        self.row_count() == self.column_count()
    }

    pub fn gaussian_elim(&mut self) {
        for r in 0..self.row_count() {
            if self.data[r][r] == 0 {
                for r_below in r+1..self.row_count() {
                    if self.data[r_below][r] != 0 {
                        self.swap_rows(r, r_below);
                        break;
                    }
                }
            }
            // If we couldn't find one, the matrix is singular.
            if self.data[r][r] == 0 {
                panic!("Matrix is singular")
            }
            // Scale to 1.
            if self.data[r][r] != 1 {
                let scale = galois::div(1, self.data[r][r]);
                for c in 0..self.column_count() {
                    self.data[r][c] = galois::mul(self.data[r][c], scale);
                }
            }
            // Make everything below the 1 be a 0 by subtracting
            // a multiple of it.  (Subtraction and addition are
            // both exclusive or in the Galois field.)
            for r_below in r+1..self.row_count() {
                if self.data[r_below][r] != 0 {
                    let scale = self.data[r_below][r];
                    for c in 0..self.column_count() {
                        self.data[r_below][c] ^= galois::mul(scale,
                                                             self.data[r][c]);
                    }
                }
            }
        }

        // Now clear the part above the main diagonal.
        for d in 0..self.row_count() {
            for r_above in 0..d {
                if self.data[r_above][d] != 0 {
                    let scale = self.data[r_above][d];
                    for c in 0..self.column_count() {
                        self.data[r_above][c] ^= galois::mul(scale,
                                                             self.data[d][c]);
                    }
                }
            }
        }
    }

    pub fn invert(&self) -> Matrix {
        if !self.is_square() {
            panic!("Trying to invert a non-square matrix")
        }

        let size = self.row_count();
        let mut work = self.augment(&Self::identity(size));

        work.gaussian_elim();

        work.sub_matrix(0,
                        self.row_count(),
                        self.column_count(),
                        self.column_count() * 2)
    }

    pub fn vandermonde(rows : usize, cols : usize) -> Matrix {
        let mut result = Self::new(rows, cols);

        for r in 0..rows {
            for c in 0..cols {
                result.data[r][c] = galois::exp(r as u8, c);
            }
        }
        result
    }
}

#[macro_export]
macro_rules! matrix {
    ($rows:expr, $cols:expr) => (Matrix::new($rows, $cols));
    (
        $(
            [ $( $x:expr ),+ ]
        ),*
    ) => (
        Matrix::new_with_data(vec![ $( vec![$( $x ),*] ),* ])
    );
}

#[cfg(test)]
mod tests {
    use super::Matrix;

    #[test]
    fn test_matrix_identity() {
        let m1 = Matrix::identity(3);
        let m2 = Matrix::new_with_data(vec![vec![1, 0, 0],
                                            vec![0, 1, 0],
                                            vec![0, 0, 1]]);
        assert_eq!(m1, m2);
    }

    #[test]
    fn test_matrix_multiply() {
        let m1 = Matrix::new_with_data(vec![vec![1, 2],
                                            vec![3, 4]]);
        let m2 = Matrix::new_with_data(vec![vec![5, 6],
                                            vec![7, 8]]);
        let actual = m1.multiply(&m2);
        let expect = Matrix::new_with_data(vec![vec![11, 22],
                                                vec![19, 42]]);
        assert_eq!(actual, expect);
    }

    #[test]
    fn test_matrix_inverse() {
        {
            let m = matrix!([56, 23, 98],
                            [3, 100, 200],
                            [45, 201, 123]).invert();
            let expect = matrix!([175, 133, 33],
                                 [130, 13, 245],
                                 [112, 35, 126]);
            assert_eq!(m, expect);
        }
    }
}

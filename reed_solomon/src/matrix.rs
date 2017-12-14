use galois;

#[derive(PartialEq)]
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
        Self::new(size, size)
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
}

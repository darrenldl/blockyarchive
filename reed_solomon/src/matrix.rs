use galois;

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

    pub fn columns_count(&self) -> usize {
        self.data[0].len()
    }

    pub fn rows_count(&self) -> usize {
        self.data.len()
    }

    pub fn get(&self, r : usize, c : usize) -> u8 {
        self.data[r][c]
    }

    pub fn set(&mut self, r : usize, c : usize, val : u8) {
        self.data[r][c] = val;
    }
}

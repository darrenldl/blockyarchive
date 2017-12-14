use galois;

pub struct Matrix {
    data : Vec<Vec<u8>>
}

impl Matrix {
    fn new(rows : usize, cols : usize) -> Matrix {
        let mut data = Vec::with_capacity(rows);

        for _ in 0..rows {
            data.push(Vec::with_capacity(cols));
        }

        Matrix { data }
    }

    fn new_with_data(init_data : Vec<Vec<u8>>) -> Matrix {
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
}

fn dummy() {
    galois::add(1, 2);
}

#![allow(dead_code)]
mod galois;
mod matrix;

use matrix::Matrix;

struct ReedSolomon {
    data_shard_count   : usize,
    parity_shard_count : usize,
    total_shard_count  : usize,
    matrix             : Matrix,
    parity_rows        : Vec<Box<[u8]>>,
}

impl ReedSolomon {
    fn build_matrix(data_shards : usize, total_shards : usize) -> Matrix {
        let vandermonde = Matrix::vandermonde(data_shards, total_shards);

        let top = vandermonde.sub_matrix(0, 0, data_shards, data_shards);

        vandermonde.multiply(&top.invert())
    }

    pub fn new(data_shards : usize, parity_shards : usize) -> ReedSolomon {
        if 256 < data_shards + parity_shards {
            panic!("Too many shards, max is 256")
        }

        let total_shards = data_shards + parity_shards;
        let matrix       = Self::build_matrix(data_shards, total_shards);
        let mut parity_rows  = Vec::with_capacity(parity_shards);
        for i in 0..parity_shards {
            parity_rows.push(matrix.get_row(data_shards + i));
        }

        ReedSolomon {
            data_shard_count   : data_shards,
            parity_shard_count : parity_shards,
            total_shard_count  : total_shards,
            matrix,
            parity_rows
        }
    }

    pub fn data_shard_count(&self) -> usize {
        self.data_shard_count
    }

    pub fn parity_shard_count(&self) -> usize {
        self.parity_shard_count
    }

    pub fn total_shard_count(&self) -> usize {
        self.total_shard_count
    }

    fn check_buffer_and_sizes(&self, shards : Vec<Box<[u8]>>) {
        
    }

    //pub fn encode_parity(&self, )
}

#[cfg(test)]
mod tests {
}

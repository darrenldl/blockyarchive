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

    fn check_buffer_and_sizes(&self, shards : &Vec<Box<[u8]>>, offset : usize, byte_count : usize) {
        if shards.len() != self.total_shard_count {
            panic!("Incorrect number of shards : {}", shards.len())
        }

        let shard_length = shards[0].len();
        for shard in shards.iter() {
            if shard.len() != shard_length {
                panic!("Shards are of different sizes");
            }
        }

        if shard_length < offset + byte_count {
            panic!("Buffers too small : {}", byte_count + offset);
        }
    }

    fn code_some_shards(&self,
                        matrix_rows : &Vec<Box<[u8]>>,
                        inputs  : &[Box<[u8]>],
                        input_count : usize,
                        outputs : &mut [Box<[u8]>],
                        output_count : usize,
                        offset : usize,
                        byte_count : usize) {
        let table = &galois::MULT_TABLE;

        {
            let i_input = 0;
            let input_shard = &inputs[i_input];
            for i_output in 0..output_count {
                let output_shard   = &mut outputs[i_output];
                let matrix_row     = &matrix_rows[i_output];
                let mult_table_row = table[matrix_row[i_input] as usize];
                for i_byte in offset..(offset + byte_count) {
                    output_shard[i_byte] = mult_table_row[input_shard[i_byte] as usize];
                }
            }
        }

        for i_input in 1..input_count {
            let input_shard = &inputs[i_input];
            for i_output in 0..output_count {
                let output_shard = &mut outputs[i_output];
                let matrix_row   = &matrix_rows[i_output];
                let mult_table_row = &table[matrix_row[i_input] as usize];
                for i_byte in offset..(offset + byte_count) {
                    output_shard[i_byte] ^= mult_table_row[input_shard[i_byte] as usize];
                }
            }
        }
    }

    pub fn encode_parity(&self, shards : &mut Vec<Box<[u8]>>, offset : usize, byte_count : usize) {
        self.check_buffer_and_sizes(shards, offset, byte_count);

        let (inputs, outputs) = shards.split_at_mut(self.data_shard_count);

        self.code_some_shards(&self.parity_rows,
                              inputs,  self.data_shard_count,
                              outputs, self.parity_shard_count,
                              offset,  byte_count);
    }

}

#[cfg(test)]
mod tests {
}

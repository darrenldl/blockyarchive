#![allow(dead_code)]
mod galois;
mod matrix;

use matrix::Matrix;

#[derive(Debug)]
pub enum Error {
    NotEnoughShards
}

struct ReedSolomon {
    data_shard_count   : usize,
    parity_shard_count : usize,
    total_shard_count  : usize,
    matrix             : Matrix,
    parity_rows        : Vec<Box<[u8]>>,
}

impl ReedSolomon {
    fn build_matrix(data_shards : usize, total_shards : usize) -> Matrix {
        let vandermonde = Matrix::vandermonde(total_shards, data_shards);

        let top = vandermonde.sub_matrix(0, 0, data_shards, data_shards);

        vandermonde.multiply(&top.invert().unwrap())
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
            panic!("Buffers too small, shard_length : {}, offset + byte_count : {}", shard_length, offset + byte_count);
        }
    }

    fn code_some_shards(matrix_rows : &Vec<Box<[u8]>>,
                        inputs  : &[Box<[u8]>],
                        outputs : &mut [Box<[u8]>],
                        offset : usize,
                        byte_count : usize) {
        let table = &galois::MULT_TABLE;

        {
            let i_input = 0;
            let input_shard = &inputs[i_input];
            for i_output in 0..outputs.len() {
                let output_shard   = &mut outputs[i_output];
                let matrix_row     = &matrix_rows[i_output];
                let mult_table_row = table[matrix_row[i_input] as usize];
                for i_byte in offset..offset + byte_count {
                    output_shard[i_byte] = mult_table_row[input_shard[i_byte] as usize];
                }
            }
        }

        for i_input in 1..inputs.len() {
            let input_shard = &inputs[i_input];
            for i_output in 0..outputs.len() {
                let output_shard = &mut outputs[i_output];
                let matrix_row   = &matrix_rows[i_output];
                let mult_table_row = &table[matrix_row[i_input] as usize];
                for i_byte in offset..offset + byte_count {
                    output_shard[i_byte] ^= mult_table_row[input_shard[i_byte] as usize];
                }
            }
        }
    }

    pub fn encode_parity(&self, shards : &mut Vec<Box<[u8]>>, offset : usize, byte_count : usize) {
        self.check_buffer_and_sizes(shards, offset, byte_count);

        let (inputs, outputs) = shards.split_at_mut(self.data_shard_count);

        Self::code_some_shards(&self.parity_rows,
                               inputs,
                               outputs,
                               offset, byte_count);
    }

    fn check_some_shards(matrix_rows : &Vec<Box<[u8]>>,
                         inputs      : &[Box<[u8]>],
                         to_check    : &[Box<[u8]>],
                         offset      : usize,
                         byte_count  : usize)
                         -> bool {
        let table = &galois::MULT_TABLE;

        for i_byte in offset..offset + byte_count {
            for i_output in 0..to_check.len() {
                let matrix_row = &matrix_rows[i_output as usize];
                let mut value = 0;
                for i_input in 0..inputs.len() {
                    value ^=
                        table
                        [matrix_row[i_input]     as usize]
                        [inputs[i_input][i_byte] as usize];
                }
                if to_check[i_output][i_byte] != value {
                    return false
                }
            }
        }
        true
    }

    pub fn is_parity_correct(&self,
                             shards : &mut Vec<Box<[u8]>>,
                             offset     : usize,
                             byte_count : usize) -> bool {
        self.check_buffer_and_sizes(shards, offset, byte_count);

        let (data_shards, to_check) = shards.split_at(self.data_shard_count);

        Self::check_some_shards(&self.parity_rows,
                                data_shards,
                                to_check,
                                offset, byte_count)
    }

    pub fn decode_missing(&self,
                          shards        : &mut Vec<Box<[u8]>>,
                          shard_present : &Vec<bool>,
                          offset : usize,
                          byte_count : usize) -> Result<(), Error>{
        self.check_buffer_and_sizes(shards, offset, byte_count);

        // Quick check: are all of the shards present?  If so, there's
        // nothing to do.
        let mut number_present = 0;
        for v in shard_present.iter() {
            if *v { number_present += 1; }
        }
        if number_present == self.total_shard_count {
            // Cool.  All of the shards data data.  We don't
            // need to do anything.
            return Ok(())
        }

        // More complete sanity check
        if number_present < self.data_shard_count {
            return Err(Error::NotEnoughShards)
        }

        // Pull out the rows of the matrix that correspond to the
        // shards that we have and build a square matrix.  This
        // matrix could be used to generate the shards that we have
        // from the original data.
        //
        // Also, pull out an array holding just the shards that
        // correspond to the rows of the submatrix.  These shards
        // will be the input to the decoding process that re-creates
        // the missing data shards.
        let mut sub_matrix =
            Matrix::new(self.data_shard_count, self.data_shard_count);
        let mut sub_shards = Vec::with_capacity(self.data_shard_count);
        {
            let mut sub_matrix_row = 0;
            let mut matrix_row = 0;
            while  matrix_row     < self.total_shard_count
                && sub_matrix_row < self.data_shard_count
            {
                if shard_present[matrix_row] {
                    for c in 0..self.data_shard_count {
                        sub_matrix.set(sub_matrix_row, c,
                                       self.matrix.get(matrix_row, c));
                    }
                    sub_shards[sub_matrix_row] = shards[matrix_row].clone();
                    sub_matrix_row += 1;
                }

                matrix_row += 1;
            }
        }

        // Invert the matrix, so we can go from the encoded shards
        // back to the original data.  Then pull out the row that
        // generates the shard that we want to decode.  Note that
        // since this matrix maps back to the orginal data, it can
        // be used to create a data shard, but not a parity shard.
        let data_decode_matrix = sub_matrix.invert().unwrap();

        // Re-create any data shards that were missing.
        //
        // The input to the coding is all of the shards we actually
        // have, and the output is the missing data shards.  The computation
        // is done using the special decode matrix we just built.
        let mut outputs = Vec::with_capacity(self.parity_shard_count);
        let mut matrix_rows  = Vec::with_capacity(self.parity_shard_count);
        let mut output_count = 0;
        for i_shard in 0..self.data_shard_count {
            if !shard_present[i_shard] {
                outputs[output_count]     = shards[i_shard].clone();
                matrix_rows[output_count] = data_decode_matrix.get_row(i_shard);
                output_count += 1;
            }
        }
        Self::code_some_shards(&matrix_rows,
                               &sub_shards,
                               outputs.as_mut_slice(),
                               offset, byte_count);

        // Now that we have all of the data shards intact, we can
        // compute any of the parity that is missing.
        //
        // The input to the coding is ALL of the data shards, including
        // any that we just calculated.  The output is whichever of the
        // data shards were missing.
        let mut output_count = 0;
        for i_shard in self.data_shard_count..self.total_shard_count {
            if !shard_present[i_shard] {
                outputs[output_count]     = shards[i_shard].clone();
                matrix_rows[output_count] =
                    self.parity_rows[i_shard
                                     - self.data_shard_count].clone();
                output_count += 1;
            }
        }
        Self::code_some_shards(&matrix_rows,
                               shards,
                               outputs.as_mut_slice(),
                               offset, byte_count);

        Ok (())
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use super::*;

    fn is_increasing_and_contains_data_row(indices : &Vec<usize>) -> bool {
        let cols = indices.len();
        for i in 0..cols-1 {
            if indices[i] >= indices[i+1] {
                return false
            }
        }
        return indices[0] < cols
    }

    fn increment_indices(indices : &mut Vec<usize>,
                         index_bound : usize) -> bool {
        for i in (0..indices.len()).rev() {
            indices[i] += 1;
            if indices[i] < index_bound {
                break;
            }

            if i == 0 {
                return false
            }

            indices[i] = 0
        }

        return true
    }

    fn increment_indices_until_increasing_and_contains_data_row(indices : &mut Vec<usize>, max_index : usize) -> bool {
        loop {
            let valid = increment_indices(indices, max_index);
            if !valid {
                return false
            }

            if is_increasing_and_contains_data_row(indices) {
                return true
            }
        }
    }

    fn find_singular_sub_matrix(m : Matrix) -> Option<Matrix> {
        let rows = m.row_count();
        let cols = m.column_count();
        let mut row_indices = Vec::with_capacity(cols);
        while increment_indices_until_increasing_and_contains_data_row(&mut row_indices, rows) {
            let mut sub_matrix = Matrix::new(cols, cols);
            for i in 0..row_indices.len() {
                let r = row_indices[i];
                for c in 0..cols {
                    sub_matrix.set(i, c, m.get(r, c));
                }
            }

            match sub_matrix.invert() {
                Err(matrix::Error::SingularMatrix) => return Some(sub_matrix),
                whatever => whatever.unwrap()
            };
        }
        None
    }

    fn fill_random(arr : &mut Box<[u8]>) {
        for a in arr.iter_mut() {
            *a = rand::random::<u8>();
        }
    }

    #[test]
    fn test_encoding() {
        let per_shard = 50_000;
        //let rng = rand::thread_rng();

        let r = ReedSolomon::new(10, 3);

        let mut shards = Vec::with_capacity(13);
        for _ in 0..13 {
            shards.push(vec![0; per_shard].into_boxed_slice());
        }

        for s in shards.iter_mut() {
            fill_random(s);
        }

        r.encode_parity(&mut shards, 0, per_shard);
        assert!(r.is_parity_correct(&mut shards, 0, per_shard));
    }
}

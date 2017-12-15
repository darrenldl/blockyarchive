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

    fn calc_offset(offset : Option<usize>) -> usize {
        match offset {
            Some(x) => x,
            None    => 0
        }
    }

    fn calc_byte_count(shards : &[Box<[u8]>], byte_count : Option<usize>) -> usize {
        match byte_count {
            Some(x) => x,
            None    => shards[0].len()
        }
    }

    fn calc_byte_count_option_shards(shards : &[Option<Box<[u8]>>], byte_count : Option<usize>) -> usize {
        match byte_count {
            Some(x) => x,
            None    => {
                for v in shards.iter() {
                    match *v {
                        Some(ref x) => return x.len(),
                        None    => {},
                    }
                };
                0
            }
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

    fn check_buffer_and_sizes_option_shards(&self,
                                            shards : &Vec<Option<Box<[u8]>>>,
                                            offset : usize, byte_count : usize) {
        if shards.len() != self.total_shard_count {
            panic!("Incorrect number of shards : {}", shards.len())
        }

        let mut shard_length = None;
        for shard in shards.iter() {
            if let Some(ref s) = *shard {
                match shard_length {
                    None    => shard_length = Some(s.len()),
                    Some(x) => {
                        if s.len() != x {
                            panic!("Shards are of different sizes");
                        }
                    }
                }
            }
        }

        if let Some(x) = shard_length {
            if x < offset + byte_count {
                panic!("Buffers too small, shard_length : Some({}), offset + byte_count : {}", x, offset + byte_count);
            }
        }
    }

    fn code_some_shards(matrix_rows  : &Vec<Box<[u8]>>,
                        inputs       : &[Box<[u8]>],
                        input_count  : usize,
                        outputs      : &mut [Box<[u8]>],
                        output_count : usize,
                        offset       : usize,
                        byte_count   : usize) {
        let table = &galois::MULT_TABLE;

        {
            let i_input = 0;
            let input_shard = &inputs[i_input];
            for i_output in 0..output_count {
                let output_shard   = &mut outputs[i_output];
                let matrix_row     = &matrix_rows[i_output];
                let mult_table_row = table[matrix_row[i_input] as usize];
                for i_byte in offset..offset + byte_count {
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
                for i_byte in offset..offset + byte_count {
                    output_shard[i_byte] ^= mult_table_row[input_shard[i_byte] as usize];
                }
            }
        }
    }

    pub fn encode_parity(&self,
                         shards     : &mut Vec<Box<[u8]>>,
                         offset     : Option<usize>,
                         byte_count : Option<usize>) {
        let offset     = Self::calc_offset(offset);
        let byte_count = Self::calc_byte_count(shards, byte_count);

        self.check_buffer_and_sizes(shards, offset, byte_count);

        let (inputs, outputs) = shards.split_at_mut(self.data_shard_count);

        Self::code_some_shards(&self.parity_rows,
                               inputs,  self.data_shard_count,
                               outputs, self.parity_shard_count,
                               offset, byte_count);
    }

    fn check_some_shards(matrix_rows : &Vec<Box<[u8]>>,
                         inputs      : &[Box<[u8]>],
                         input_count : usize,
                         to_check    : &[Box<[u8]>],
                         check_count : usize,
                         offset      : usize,
                         byte_count  : usize)
                         -> bool {
        let table = &galois::MULT_TABLE;

        for i_byte in offset..offset + byte_count {
            for i_output in 0..check_count {
                let matrix_row = &matrix_rows[i_output as usize];
                let mut value = 0;
                for i_input in 0..input_count {
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
                             shards     : &Vec<Box<[u8]>>,
                             offset     : Option<usize>,
                             byte_count : Option<usize>) -> bool {
        let offset     = Self::calc_offset(offset);
        let byte_count = Self::calc_byte_count(shards, byte_count);

        self.check_buffer_and_sizes(shards, offset, byte_count);

        let (data_shards, to_check) = shards.split_at(self.data_shard_count);

        Self::check_some_shards(&self.parity_rows,
                                data_shards, self.data_shard_count,
                                to_check,    self.parity_shard_count,
                                offset, byte_count)
    }

    pub fn shards_to_option_shards(shards : &Vec<Box<[u8]>>)
                                   -> Vec<Option<Box<[u8]>>> {
        let mut result = Vec::with_capacity(shards.len());

        for v in shards.iter() {
            result.push(Some(v.clone()));
        }
        result
    }

    pub fn option_shards_to_shards(shards : &Vec<Option<Box<[u8]>>>,
                                   offset : usize,
                                   count : usize)
                                   -> Vec<Box<[u8]>> {
        let mut result = Vec::with_capacity(shards.len());

        for i in offset..offset+count {
            let shard = match shards[i] {
                Some(ref x) => x,
                None        => panic!("Missing shards"),
            };
            result.push(shard.clone());
        }
        result
    }

    pub fn decode_missing(&self,
                          shards        : &mut Vec<Option<Box<[u8]>>>,
                          offset        : Option<usize>,
                          byte_count    : Option<usize>)
                          -> Result<(), Error> {
        let offset     = Self::calc_offset(offset);
        let byte_count = Self::calc_byte_count_option_shards(shards, byte_count);

        self.check_buffer_and_sizes_option_shards(shards, offset, byte_count);

        // Quick check: are all of the shards present?  If so, there's
        // nothing to do.
        let mut number_present = 0;
        for v in shards.iter() {
            if let Some(_) = *v { number_present += 1; }
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
        let mut sub_shards : Vec<Box<[u8]>> =
            vec![Box::new([0]); self.data_shard_count];
        {
            let mut sub_matrix_row = 0;
            let mut matrix_row = 0;
            while  matrix_row     < self.total_shard_count
                && sub_matrix_row < self.data_shard_count
            {
                if let Some(_) = shards[matrix_row] {
                    for c in 0..self.data_shard_count {
                        sub_matrix.set(sub_matrix_row, c,
                                       self.matrix.get(matrix_row, c));
                    }
                    sub_shards[sub_matrix_row] =
                        shards[matrix_row].clone().unwrap();
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
        let mut outputs : Vec<Box<[u8]>> =
            vec![vec![0; byte_count].into_boxed_slice();
                 self.parity_shard_count];
        let mut matrix_rows  : Vec<Box<[u8]>> =
            vec![Box::new([]); self.parity_shard_count];
        let mut output_count = 0;
        for i_shard in 0..self.data_shard_count {
            if let None = shards[i_shard] {
                matrix_rows[output_count] = data_decode_matrix.get_row(i_shard);
                output_count += 1;
            }
        }
        Self::code_some_shards(&matrix_rows,
                               &sub_shards,  self.data_shard_count,
                               &mut outputs, output_count,
                               offset, byte_count);

        // copy outputs to slots with missing shards
        let mut output_count = 0;
        for i_shard in 0..self.data_shard_count {
            if let None = shards[i_shard] {
                shards[i_shard] = Some(outputs[output_count].clone());
                output_count += 1;
            }
        }

        // Now that we have all of the data shards intact, we can
        // compute any of the parity that is missing.
        //
        // The input to the coding is ALL of the data shards, including
        // any that we just calculated.  The output is whichever of the
        // data shards were missing.
        let mut outputs : Vec<Box<[u8]>> =
            vec![vec![0u8; byte_count].into_boxed_slice();
                 self.parity_shard_count];
        let mut output_count = 0;
        for i_shard in self.data_shard_count..self.total_shard_count {
            if let None = shards[i_shard] {
                matrix_rows[output_count] =
                    self.parity_rows[i_shard
                                     - self.data_shard_count].clone();
                output_count += 1;
            }
        }
        let complete_data_shards =
            Self::option_shards_to_shards(shards, 0, self.data_shard_count);
        Self::code_some_shards(&matrix_rows,
                               &complete_data_shards, self.data_shard_count,
                               outputs.as_mut_slice(), output_count,
                               offset, byte_count);

        // copy outputs to parity shards slots
        let mut output_count = 0;
        for i_shard in self.data_shard_count..self.total_shard_count {
            if let None = shards[i_shard] {
                shards[i_shard] = Some(outputs[output_count].clone());
                output_count += 1;
            }
        }

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

        let r = ReedSolomon::new(10, 3);

        let mut shards = Vec::with_capacity(13);
        for _ in 0..13 {
            shards.push(vec![0; per_shard].into_boxed_slice());
        }

        for s in shards.iter_mut() {
            fill_random(s);
        }

        r.encode_parity(&mut shards, None, None);
        assert!(r.is_parity_correct(&shards, None, None));
    }

    #[test]
    fn test_decode_missing() {
        let per_shard = 50_000;

        let r = ReedSolomon::new(10, 3);

        let mut shards = Vec::with_capacity(13);
        for _ in 0..13 {
            shards.push(vec![0; per_shard].into_boxed_slice());
        }

        for s in shards.iter_mut() {
            fill_random(s);
        }

        let shards = ReedSolomon::shards_to_option_shards(&shards);

        /*r.encode_parity(&mut shards, None, None);

        // Try to decode with all shards present
        r.decode_missing(&mut shards,
                         None, None).unwrap();
        assert!(r.is_parity_correct(&shards, None, None));

        // Try to decode with 10 shards
        r.decode_missing(&mut shards,
                         None, None).unwrap();
        assert!(r.is_parity_correct(&shards, None, None));

	      // Try to deocde with 6 data and 4 parity shards
        r.decode_missing(&mut shards,
                         None, None).unwrap();
        assert!(r.is_parity_correct(&shards, None, None));*/
    }
}

#![cfg(test)]

use super::*;

#[test]
fn test_calc_parity_shards_simple_cases() {
    {
        let data_shards   = 10;
        let parity_shards = 2;

        assert_eq!(1, calc_parity_shards(data_shards,
                                         parity_shards,
                                         1));
        assert_eq!(1, calc_parity_shards(data_shards,
                                         parity_shards,
                                         2));
        assert_eq!(1, calc_parity_shards(data_shards,
                                         parity_shards,
                                         3));
        assert_eq!(1, calc_parity_shards(data_shards,
                                         parity_shards,
                                         4));
        assert_eq!(1, calc_parity_shards(data_shards,
                                         parity_shards,
                                         5));
        assert_eq!(2, calc_parity_shards(data_shards,
                                         parity_shards,
                                         6));
        assert_eq!(2, calc_parity_shards(data_shards,
                                         parity_shards,
                                         7));
        assert_eq!(2, calc_parity_shards(data_shards,
                                         parity_shards,
                                         8));
        assert_eq!(2, calc_parity_shards(data_shards,
                                         parity_shards,
                                         9));
        assert_eq!(2, calc_parity_shards(data_shards,
                                         parity_shards,
                                         10));
        assert_eq!(3, calc_parity_shards(data_shards,
                                         parity_shards,
                                         11));
        assert_eq!(3, calc_parity_shards(data_shards,
                                         parity_shards,
                                         12));
    }
}

mod from_data_block_count {
    use super::super::from_data_block_count::*;
    use super::super::super::sbx_specs::SBX_RS_ENABLED_FIRST_DATA_SEQ_NUM;

    #[test]
    fn test_last_data_set_start_index_simple_cases() {
        let data_shards = 10;

        assert_eq!(0, last_data_set_start_index(data_shards,
                                                0));
        assert_eq!(0, last_data_set_start_index(data_shards,
                                                10));
        assert_eq!(10, last_data_set_start_index(data_shards,
                                                 11));
        assert_eq!(10, last_data_set_start_index(data_shards,
                                                 20));
        assert_eq!(20, last_data_set_start_index(data_shards,
                                                 21));
    }

    #[test]
    fn test_last_data_set_size() {
        let data_shards = 11;

        assert_eq!(0, last_data_set_size(data_shards,
                                         0));
        assert_eq!(1, last_data_set_size(data_shards,
                                         1));
        assert_eq!(11, last_data_set_size(data_shards,
                                          11));
        assert_eq!(1, last_data_set_size(data_shards,
                                         12));
        assert_eq!(11, last_data_set_size(data_shards,
                                          22));
    }

    #[test]
    fn test_last_block_set_start_seq_num_simple_cases() {
        let data_shards   = 7;
        let parity_shards = 3;

        assert_eq!(SBX_RS_ENABLED_FIRST_DATA_SEQ_NUM as u32,
                   last_block_set_start_seq_num(data_shards,
                                                parity_shards,
                                                0));
        assert_eq!(SBX_RS_ENABLED_FIRST_DATA_SEQ_NUM as u32,
                   last_block_set_start_seq_num(data_shards,
                                                parity_shards,
                                                1));
        assert_eq!(SBX_RS_ENABLED_FIRST_DATA_SEQ_NUM as u32 + 10,
                   last_block_set_start_seq_num(data_shards,
                                                parity_shards,
                                                11));
        assert_eq!(SBX_RS_ENABLED_FIRST_DATA_SEQ_NUM as u32 + 10,
                   last_block_set_start_seq_num(data_shards,
                                                parity_shards,
                                                13));
        assert_eq!(SBX_RS_ENABLED_FIRST_DATA_SEQ_NUM as u32 + 10,
                   last_block_set_start_seq_num(data_shards,
                                                parity_shards,
                                                14));
        assert_eq!(SBX_RS_ENABLED_FIRST_DATA_SEQ_NUM as u32 + 20,
                   last_block_set_start_seq_num(data_shards,
                                                parity_shards,
                                                15));
    }

    #[test]
    fn test_last_block_set_size_simple_cases() {
        let data_shards   = 9;
        let parity_shards = 2;

        assert_eq!(0, last_block_set_size(data_shards,
                                          parity_shards,
                                          0));
        assert_eq!(1 + 1, last_block_set_size(data_shards,
                                              parity_shards,
                                              1));
        assert_eq!(1 + 1, last_block_set_size(data_shards,
                                              parity_shards,
                                              10));
        assert_eq!(9 + 2, last_block_set_size(data_shards,
                                              parity_shards,
                                              9));
        assert_eq!(9 + 2, last_block_set_size(data_shards,
                                              parity_shards,
                                              18));
    }

    #[test]
    fn test_calc_total_blocks_simple_cases() {
        let data_shards = 3;
        let parity_shards = 2;

        assert_eq!(4 + 2, calc_total_blocks(data_shards,
                                            parity_shards,
                                            1));
        assert_eq!(4 + 5, calc_total_blocks(data_shards,
                                            parity_shards,
                                            3));
        assert_eq!(4 + 5 + 2, calc_total_blocks(data_shards,
                                                parity_shards,
                                                4));
        assert_eq!(4 + 5 + 4, calc_total_blocks(data_shards,
                                                parity_shards,
                                                5));
        assert_eq!(4 + 10, calc_total_blocks(data_shards,
                                             parity_shards,
                                             6));
    }

    #[test]
    fn test_seq_num_is_parity_simple_cases() {
        let data_shards       = 5;
        let parity_shards     = 1;
        let total_data_chunks = 21;

        assert_eq!(false, seq_num_is_parity(0,
                                            data_shards,
                                            parity_shards,
                                            total_data_chunks));
        assert_eq!(true,  seq_num_is_parity(1,
                                            data_shards,
                                            parity_shards,
                                            total_data_chunks));
        assert_eq!(true,  seq_num_is_parity(2,
                                            data_shards,
                                            parity_shards,
                                            total_data_chunks));
        assert_eq!(true,  seq_num_is_parity(3,
                                            data_shards,
                                            parity_shards,
                                            total_data_chunks));
        assert_eq!(false, seq_num_is_parity(4,
                                            data_shards,
                                            parity_shards,
                                            total_data_chunks));
        assert_eq!(true,  seq_num_is_parity(9,
                                            data_shards,
                                            parity_shards,
                                            total_data_chunks));
        assert_eq!(false, seq_num_is_parity(10,
                                            data_shards,
                                            parity_shards,
                                            total_data_chunks));
        assert_eq!(true,  seq_num_is_parity(15,
                                            data_shards,
                                            parity_shards,
                                            total_data_chunks));
        assert_eq!(false, seq_num_is_parity(16,
                                            data_shards,
                                            parity_shards,
                                            total_data_chunks));
        assert_eq!(true,  seq_num_is_parity(21,
                                            data_shards,
                                            parity_shards,
                                            total_data_chunks));
        assert_eq!(true,  seq_num_is_parity(27,
                                            data_shards,
                                            parity_shards,
                                            total_data_chunks));
        assert_eq!(false, seq_num_is_parity(28,
                                            data_shards,
                                            parity_shards,
                                            total_data_chunks));
        assert_eq!(true,  seq_num_is_parity(29,
                                            data_shards,
                                            parity_shards,
                                            total_data_chunks));
    }
}

mod from_total_block_count {
    use super::super::from_total_block_count::*;
    use super::super::super::sbx_specs::SBX_RS_ENABLED_FIRST_DATA_SEQ_NUM;

    #[test]
    fn test_last_block_set_start_seq_num_simple_cases() {
        let data_shards = 2;
        let parity_shards = 3;

        assert_eq!(SBX_RS_ENABLED_FIRST_DATA_SEQ_NUM as u32 + 5,
                   last_block_set_start_seq_num(data_shards,
                                                parity_shards,
                                                14));
        assert_eq!(SBX_RS_ENABLED_FIRST_DATA_SEQ_NUM as u32 + 10,
                   last_block_set_start_seq_num(data_shards,
                                                parity_shards,
                                                16));
    }

    #[test]
    fn test_last_block_set_size_simple_cases() {
        let data_shards = 2;
        let parity_shards = 3;

        assert_eq!(5, last_block_set_size(data_shards,
                                          parity_shards,
                                          14));
        assert_eq!(2, last_block_set_size(data_shards,
                                          parity_shards,
                                          16));
        assert_eq!(3, last_block_set_size(data_shards,
                                          parity_shards,
                                          17));
    }
}

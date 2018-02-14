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
}

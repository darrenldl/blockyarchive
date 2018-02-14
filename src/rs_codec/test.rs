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

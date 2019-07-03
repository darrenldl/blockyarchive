#![cfg(test)]
use super::*;
use proptest::prelude::*;
use crate::sbx_specs::{Version};

#[test]
#[should_panic]
fn cancel_last_slot_when_empty1() {
    let mut lot = Lot::new(Version::V17,
                           None,
                           InputType::Block,
                           OutputType::Block,
                           BlockArrangement::Unordered,
                           None,
                           true,
                           10,
                           false,
                           &Arc::new(None),
    );

    lot.cancel_last_slot();
}

#[test]
#[should_panic]
fn cancel_last_slot_when_empty2() {
    let mut lot = Lot::new(Version::V17,
                       None,
                       InputType::Block,
                       OutputType::Block,
                       BlockArrangement::Unordered,
                       None,
                       true,
                       10,
                       false,
                       &Arc::new(None),
    );

    let _ = lot.get_slot();

    lot.cancel_last_slot();
    lot.cancel_last_slot();
}

proptest! {
    #[test]
    #[should_panic]
    fn pt_cancel_last_slot_when_empty(size in 0usize..1000) {
        let mut lot = Lot::new(Version::V17,
                               None,
                               InputType::Block,
                               OutputType::Block,
                               BlockArrangement::Unordered,
                               None,
                               true,
                               size,
                               false,
                               &Arc::new(None),
        );

        for _ in 0..size {
            let _ = lot.get_slot();
        }

        for _ in 0..size+1 {
            lot.cancel_last_slot();
        }
    }

    #[test]
    fn pt_cancel_last_slot_when_not_empty(size in 0usize..1000) {
        let mut lot = Lot::new(Version::V17,
                               None,
                               InputType::Block,
                               OutputType::Block,
                               BlockArrangement::Unordered,
                               None,
                               true,
                               size,
                               false,
                               &Arc::new(None),
        );

        for _ in 0..size {
            let _ = lot.get_slot();
        }

        for _ in 0..size {
            lot.cancel_last_slot();
        }
    }
}

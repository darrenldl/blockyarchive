#![cfg(test)]
use super::*;
use proptest::prelude::*;
use crate::sbx_specs::{Version};
use crate::multihash::hash;
use crate::multihash::{HashType};

#[test]
#[should_panic]
fn cancel_last_slot_panics_when_empty1() {
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
fn cancel_last_slot_panics_when_empty2() {
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

#[test]
fn hash_when_correct_arrangment1() {
    let lot = Lot::new(Version::V17,
                       None,
                       InputType::Block,
                       OutputType::Block,
                       BlockArrangement::OrderedAndNoMissing,
                       None,
                       true,
                       10,
                       false,
                       &Arc::new(None),
    );

    let mut hash_ctx = hash::Ctx::new(HashType::SHA256).unwrap();

    lot.hash(&mut hash_ctx);
}

#[test]
fn hash_when_correct_arrangment2() {
    let lot = Lot::new(Version::V17,
                       None,
                       InputType::Block,
                       OutputType::Block,
                       BlockArrangement::OrderedButSomeMayBeMissing,
                       None,
                       true,
                       10,
                       false,
                       &Arc::new(None),
    );

    let mut hash_ctx = hash::Ctx::new(HashType::SHA256).unwrap();

    lot.hash(&mut hash_ctx);
}

#[test]
#[should_panic]
fn hash_panics_when_incorrect_arrangement() {
    let lot = Lot::new(Version::V17,
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

    let mut hash_ctx = hash::Ctx::new(HashType::SHA256).unwrap();

    lot.hash(&mut hash_ctx);
}

proptest! {
    #[test]
    #[should_panic]
    fn pt_cancel_last_slot_panics_when_empty(size in 1usize..1000) {
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
    fn pt_cancel_last_slot_when_not_empty(size in 1usize..1000) {
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

    #[test]
    fn pt_get_slot_result(size in 1usize..1000) {
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

        for _ in 0..size-1 {
            match lot.get_slot() {
                GetSlotResult::None => panic!(),
                GetSlotResult::Some(_, _, _) => {},
                GetSlotResult::LastSlot(_, _, _) => panic!(),
            }
        }

        match lot.get_slot() {
            GetSlotResult::None => panic!(),
            GetSlotResult::Some(_, _, _) => panic!(),
            GetSlotResult::LastSlot(_, _, _) => {},
        }

        match lot.get_slot() {
            GetSlotResult::None => {},
            GetSlotResult::Some(_, _, _) => panic!(),
            GetSlotResult::LastSlot(_, _, _) => panic!(),
        }
    }
}

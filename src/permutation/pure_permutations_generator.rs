//todo: may need to inc to u64 for higher n

use crate::permutation::permutation_label::get_all_pure_lehmer_codes_of_size;

struct PermutationTracker {
    pure_lehmer_codes_for_size: Vec<Vec<u32>>
}

impl PermutationTracker {
    fn upto_size(size: u8) -> PermutationTracker {
        let mut pure_lehmer_codes_upto_size: Vec<Vec<u32>> = vec![];
        for n in 0..=size {
            let pure_lehmer_codes_for_n = get_all_pure_lehmer_codes_of_size(n);
            pure_lehmer_codes_upto_size.push(pure_lehmer_codes_for_n);
        }
        return PermutationTracker {pure_lehmer_codes_for_size: pure_lehmer_codes_upto_size };
    }
}


#[test]
fn test_pure_permutation_tracker_upto_size() {
    let permutation_tracker = PermutationTracker::upto_size(5);

    println!("Permutation Trackers : {:?}", permutation_tracker.pure_lehmer_codes_for_size);
}
use std::collections::HashMap;

struct PermutationMap {
    permutation_to_label: HashMap<Vec<i8>, i64>,
    label_to_permutation: HashMap<i64, Vec<i8>>
}
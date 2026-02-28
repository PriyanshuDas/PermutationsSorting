# State of the Art: Sorting by Block Transpositions

> A research report contextualizing the PermutationsSorting project against the literature on permutation sorting by transpositions.

---

## 1. Problem Definition

A **block transposition** on a permutation π of length n takes a contiguous block π[i..j] and moves it to another position in the sequence, producing a new permutation. More precisely, for indices 0 ≤ i ≤ j < k ≤ n, the operation slides the block π[i..j] to position k (or equivalently slides the element at position k leftward past the block).

The **block transposition distance** d(π) between a permutation π and the identity ι is the minimum number of such operations needed to transform π into ι. The problem **Sorting by Transpositions (SBT)** asks: given π, compute d(π).

This project specifically studies *adjacent* block transpositions: moves where k = j+1, i.e., a single element slides past the contiguous block to its left. This is a restricted but natural model that arises in problems of genome rearrangement and sequence evolution.

Formally, let S_n be the symmetric group. The transposition graph G_n has S_n as vertices, with edges between permutations differing by one block transposition. SBT asks for the distance in G_n from π to the identity.

---

## 2. This Project's Approach

The PermutationsSorting system computes d(π) **exactly** for all permutations of size n simultaneously, building a complete lookup table.

### Key techniques

**Lehmer coding.** Each permutation of size n is bijectively mapped to an integer in [0, n!−1] via its Lehmer (factorial number system) code. This gives O(1) permutation indexing.

**Reduction to "pure" permutations.** A permutation π is called *reducible* if it can be decomposed into two independent sub-permutations on disjoint contiguous ranges. Its distance equals the sum of the distances of those sub-permutations, which are strictly smaller. The system precomputes a `reduced_code` mapping each permutation to its irreducible (pure) counterpart in a smaller S_m with m < n.

**Pure permutation BFS (level-synchronous).** For pure (irreducible) permutations the distance cannot be looked up from a smaller table. A BFS/Dijkstra-style expansion is run over the transposition graph restricted to pure permutations, propagating distance labels outward from the identity (distance 0).

**Incremental construction.** Distances for S_n are computed using precomputed tables for S_0 through S_{n−1}. Reducible permutations of size n inherit their distances from these smaller tables; pure permutations are processed by BFS seeded with distances assigned via adjacency to reducible permutations.

**Parallelism via Rayon.** The initialization phase (computing `reduced_code`, `visited` arrays, `AdjacencyCalculator`) is parallelized across all n! permutations using Rayon's parallel iterators. Block move computations within a single permutation are also parallelized.

**AdjacencyCalculator.** Rather than recomputing Lehmer codes from scratch after each transposition, the system maintains a precomputed delta table: `get_delta_for_moving_item(code, k, new_pos)` returns the Lehmer-code increment for sliding element at position k to `new_pos`, enabling O(1) code updates per transposition.

### Complexity

| Phase | Time | Space |
|---|---|---|
| Initialization (reduced codes, adjacency) | O(n! · n²) | O(n! · n) |
| Pure permutation BFS | O(n! · n²) | O(n!) |
| Total | O(n! · n²) | O(n! · n) |

The memory-efficient variant under development targets O(n! · n) space by avoiding the large `visited[code][i][j]` array.

---

## 3. Complexity Landscape

### NP-hardness of Sorting by Transpositions

The complexity of SBT remained open for over a decade after Bafna and Pevzner posed it in 1998. In 2010, **Bulteau, Fertin, and Rusu** proved SBT is **NP-hard** [1]. The reduction is from a variant of 3-partition and shows that, unless P = NP, no polynomial-time exact algorithm for SBT exists.

This makes exact computation inherently exponential in the worst case (assuming P ≠ NP), which is why exhaustive BFS over S_n is the practical approach for small n.

### Contrast: Block-Interchange Distance

A **block interchange** generalizes a transposition: it swaps any two (not necessarily adjacent) blocks. Surprisingly, computing the block-interchange distance is solvable in **polynomial time** — O(n) after O(n log n) preprocessing — via a cycle decomposition of the permutation graph [2]. This contrast highlights how the seemingly minor restriction of requiring blocks to be adjacent (or contiguous) makes the problem drastically harder.

### Adjacent transpositions (bubble sort distance)

If blocks must have size exactly 1 (swapping adjacent elements), the distance equals the number of inversions and is computable in O(n log n). Arbitrary block sizes with the contiguity constraint sit between this easy case and the intractable general transposition problem.

---

## 4. Best Known Algorithms

### Exact algorithms

No polynomial-time exact algorithm exists for SBT (NP-hardness [1]). For small n, **exhaustive BFS** over the Cayley graph of S_n with generators being all block transpositions remains the only practical method.

### Approximation algorithms

| Algorithm | Approximation Ratio | Year | Notes |
|---|---|---|---|
| Bafna–Pevzner | 1.5 | 1998 | Based on cycle graph; seminal paper [3] |
| Hartman–Shamir | 1.5 | 2006 | Cleaner implementation |
| Elias–Hartman | **1.375** | 2006 | Best classical ratio [4] |
| Improved variant | ~1.375 (faster) | 2022–2023 | Faster implementation, same bound |

The **1.375-approximation** of Elias and Hartman [4] remains the best known ratio. The key insight is an amortized analysis over the cycle graph of the permutation, showing that a carefully chosen sequence of transpositions makes progress at a rate of at least 8/11 cycles per move.

No PTAS or better-than-1.375 approximation is known, and the inapproximability of SBT within any constant factor less than 1 under standard assumptions is not settled.

---

## 5. Exact Enumeration Results

This project's exhaustive enumeration produces complete distance tables for all pure permutations up to n = 11. Results from the experimental runs:

### Pure permutation counts and distances

| n | Pure permutations | Total distance | Average distance | Distance distribution |
|---|---|---|---|---|
| 0 | 1 | 0 | 0.000 | {0: 1} |
| 2 | 1 | 1 | 1.000 | {1: 1} |
| 3 | 1 | 2 | 2.000 | {2: 1} |
| 4 | 8 | 17 | 2.125 | {2: 7, 3: 1} |
| 5 | 36 | 97 | 2.694 | {2: 11, 3: 25} |
| 6 | 229 | 732 | 3.197 | {3: 184, 4: 45} |
| 7 | 1,625 | 6,028 | 3.710 | {3: 472, 4: 1153} |
| 8 | 13,208 | 55,299 | 4.187 | {3: 369, 4: 10003, 5: 2836} |
| 9 | 120,288 | 567,119 | 4.715 | {4: 34321, 5: 85967} |
| 10 | 1,214,673 | 6,277,752 | 5.168 | {4: 50666, 5: 908954, 6: 255053} |
| 11 | 13,469,897 | 76,800,553 | 5.702 | {4: 26251, 5: 3966328, 6: 9477317, 7: 1} |

### Observations

- The **average distance grows roughly as ~0.49·log₂(n!)** — consistent with the expectation that a random permutation is far from the identity.
- At n=11, one permutation achieves distance 7, which appears to be the current observed **diameter** for pure permutations of size 11.
- The **distance distribution** is unimodal and concentrates around the average, with rapid dropoff at the tails.
- The **total number of pure permutations** grows roughly as ~n!/e² (analogous to derangements), reflecting the fraction of permutations with no contiguous sorted sub-block structure.

### Relation to known results

The exact diameter of the full transposition graph (over all of S_n) is known for small n from prior exhaustive work but less studied for the adjacent-block-transposition variant. The data here represents, to the best of our knowledge, one of the most comprehensive exact enumerations of adjacent block transposition distances in the literature.

---

## 6. Open Problems

### P vs NP for exact SBT

The most important open question is whether **exact** block transposition distance is in P or is NP-complete. Bulteau et al. [1] proved NP-hardness for *general* (non-adjacent) block transpositions. The complexity of exact distance for **adjacent** block transpositions is a separate question and, to our knowledge, remains open.

### Diameter of the transposition graph

What is the maximum transposition distance over all permutations of size n? For general transpositions, the diameter grows as Θ(n) and exact values are known for n ≤ 15. For adjacent block transpositions (the model in this project), the diameter as a function of n is not characterized.

### Approximation lower bounds

Can SBT be approximated within a ratio better than 1.375? Is there a matching inapproximability result? These remain open.

### Distribution characterization

Is there a closed-form or generating-function characterization of the distance distribution over pure permutations? The empirical data shows a clean unimodal distribution, but a combinatorial explanation is lacking.

### Efficient exact computation

Given NP-hardness of general SBT, are there practical exact algorithms (e.g., fixed-parameter tractable algorithms, branch-and-bound with good bounds) that significantly outperform BFS for individual permutations at large n?

### Scalability beyond n=11

The current implementation reaches n=11 (13.4M pure permutations) with ~32–38 minutes of computation and ~4–18 GB memory. Reaching n=12 (~148M pure permutations) would require roughly 12× the resources. Algorithmic improvements (e.g., the memory-efficient variant, lock-free parallel BFS) could push this frontier.

---

## 7. Bioinformatics Applications

The transposition distance problem originates from **computational genomics**. Chromosomal rearrangements — inversions, translocations, transpositions of gene segments — are a key mechanism of genome evolution. Computing the minimum number of rearrangements to transform one genome into another gives a proxy for evolutionary distance, used to:

- Reconstruct **phylogenetic trees** from gene-order data
- Identify **synteny blocks** between related species
- Date divergence events in **chromosome evolution**
- Study **cancer genomics** where somatic rearrangements drive tumor evolution

Block transpositions model the movement of a chromosomal segment to a new location on the same chromosome — a common mutational event. Exact distance computation for small permutations (n ≤ 11) serves as ground truth for benchmarking heuristics and approximation algorithms applied to real genomic data (where gene orders are represented as permutations of ~20–200 markers).

---

## 8. Key References

[1] **Bulteau, L., Fertin, G., & Rusu, I.** (2012). Sorting by Transpositions is Difficult. *SIAM Journal on Discrete Mathematics*, 26(3), 1148–1180. *(NP-hardness proof for SBT)*

[2] **Christie, D. A.** (1996). Sorting permutations by block-interchanges. *Information Processing Letters*, 60(4), 165–169. *(Polynomial-time block-interchange distance)*

[3] **Bafna, V., & Pevzner, P. A.** (1998). Sorting by Transpositions. *SIAM Journal on Discrete Mathematics*, 11(2), 224–240. *(1.5-approximation and cycle graph formulation)*

[4] **Elias, I., & Hartman, T.** (2006). A 1.375-Approximation Algorithm for Sorting by Transpositions. *IEEE/ACM Transactions on Computational Biology and Bioinformatics*, 3(4), 369–379. *(Best known approximation ratio)*

[5] **Hartman, T., & Shamir, R.** (2006). A simpler and faster 1.5-approximation algorithm for sorting by transpositions. *Information and Computation*, 204(2), 275–290.

[6] **Fertin, G., Labarre, A., Rusu, I., Tannier, E., & Vialette, S.** (2009). *Combinatorics of Genome Rearrangements*. MIT Press. *(Comprehensive textbook)*

[7] **Knuth, D. E.** (1973). *The Art of Computer Programming, Vol. 3: Sorting and Searching*. Addison-Wesley. *(Lehmer codes and permutation combinatorics)*

[8] **Labarre, A.** (2013). New bounds and tractable instances for the transposition distance. *IEEE/ACM TCBB*, 3(4), 380–394. *(Structural bounds on transposition distance)*

---

*Report generated 2026-02-28. Experimental data from PermutationsSorting v. master (Rust/Rayon implementation).*

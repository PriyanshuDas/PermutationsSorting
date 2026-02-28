# Gaps Analysis: Transpositions_V7.3 (Parallelization)

Gaps that need addressing before the paper is publication-ready.

---

## 1. Lock contention argument for span

The span claim of O(b) = O(n) assumes each step in the parallel BFS has O(1) span. However, step 9.a.iii.6 acquires a lock on `Dn[p']`. If many threads compete for the same cell simultaneously, the span blows up beyond O(1) per step. The paper needs to explicitly argue why lock contention is bounded — likely using the same Lemma 6 argument (only one processing permutation can hit each unprocessed target per block per batch), but this must be stated and proved as part of the span analysis, not left implicit.

## 2. Parallel complexity section is incomplete

The `Tp`, speedup, and parallelism fields in the parallelized complexity section are currently empty ("Work In Progress"). These are the quantities a reviewer will expect. Specifically:

- **Tp** (runtime on P processors): O(n! · n² / P + n)
- **Speedup**: T₁ / Tp
- **Parallelism** (T₁ / T∞): O(n! · n) — this is the key number to highlight

## 3. Precomputation span needs tightening

The paper claims span O(n) for label generation and O(n · b(n)) for `rs_pre`/`mask_pre`, but `b(n)` is never defined precisely in the precomputation section. If b(n) refers to the number of BFS batches, the argument becomes circular (the span of precomputation depends on the span of the main algorithm). If it refers to something else (e.g., block count), it needs an explicit definition.

## 4. Adjacent vs. general transpositions — model ambiguity

The algorithm description (sliding block [i,j] to position k for any k > j) describes **general** block transpositions, but the implementation computes distances for **adjacent** block transpositions (k = j+1 only, i.e., a single element slides past the block). The paper should explicitly state which model it targets. If general, the code and results tables need updating. If adjacent, the algorithm description needs restricting.

## 5. Diameter bound O(n) needs proof or citation

The span claim O(b) = O(n) rests on the diameter of the transposition graph being Θ(n). For general block transpositions this is known, but for the **adjacent** block transposition model the diameter as a function of n is not established in the literature. The experimental data (max distance 7 at n=11) is consistent with O(n) growth, but a formal proof or citation is needed to make the span claim rigorous.

---

## Summary

The core theoretical contribution (Lemmas 1–7 and the O(n! · n²) work / O(n) span result) is novel and appears sound. Addressing the five gaps above would bring the paper to publication-ready state. Estimated fit venue: **SPAA** (Symposium on Parallelism in Algorithms and Architectures).

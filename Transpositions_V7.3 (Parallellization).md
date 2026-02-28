# Serialized Algorithm

1. Dn= set of distances of permutations in Sn,  
    such that i \< n\! *Dn\[i\]* \= distance estimate of permutation *p* represented by label *i*  
2. Initialize distances and visited  
   1. **for all** i, *0\<=p\< n\!*,   
      1. set *Dn\[p\] \= n*, initialize all distance estimates to *n*  
      2. **for** block *\[i,j\]* in *p* : 0ij\<(n-1)  
         1. visited\[*p*\]\[*i*\]\[*j*\] \= false 	// this is for tracking permutation, block pair  
3. Set Dn\[0\]=0, distance of In is 0  
4. B= set of *unprocessed* labels with minimum distance estimates  
5. B={0}, start with batch of only *In*, represented by label 0  
6. Let *dcurr* \= 0, distance if *In* is 0  
7. **for** **all** *p* in B, mark *p* as *processing*  
8. Let Bnext={}  
9. **for** *p* in *B*  
   1. **for** block *\[i,j\]* in *p* : 0ij\<(n-1)  
      1. size=(j-i+1)  // size of the block we are sliding  
      2. p'=p                      // The permutation reached by sliding block  
      3. for *k* : j\<k\<n  
         1. shift=(k-j)  // By how much we have slid the block  
         2. p'=p'+delta(p',k,k-size) // this is using precomp, O(1)  
         3. if *p’* is processed or processing, continue to next block in (*9.a*)  
         4. if visited\[*p’*\]\[*i+shift*\]\[*j+shift*\]: continue to next block in (9.a)  
         5. visited\[*p’*\]\[*i+shift*\]\[*j+shift*\] \= true  
         6. if Dn\[p'\]\>dcurr+1,  // if not true, then this *p’* is already in next batch  
            1. set Dn\[p'\]=dcurr+1   
            2. add *p’* to Bnext   
         7. else, continue to next block in (9.a)  
10. **for** **all** *p* in B, mark *p* as *processed*  
11. If Bnext is not empty,   
    1. set B=Bnext and *dcurr* \= *dcurr* \+ 1  
    2.  go to *7*  
12. else, end algorithm, Dnhas the distance of all labels.

## Complexity

### **Theorem**: 

For every permutation *p*, and block *b* within *p*, *p* is hit a max of three times asymptotically while sliding *b*:

1. When a *processing* permutation hits *unprocessed* permutation *p* while sliding *b*.  
2. When *processing p* by sliding block *b*.  
3. Potentially, one of the following:  
   1.  when permutation *p’* hits *p* by sliding *b* while both are *processing*.  
   2. when a *processing* permutation *p’* reaches *processed* permutation *p* by sliding *b*.

This leads to the innermost loop running O(n\!\*n2) times asymptotically. 

### **Proof**:

It is trivial to see that 2 happens only once. But it is non-trivial to why **1** and **3** happen at max once each. This is what we prove below, with the help of a few lemmas about the blocks and permutations having these blocks.

Consider a block *b*. There are distinct sets of permutations in *Sn* with block *b*, such that a permutation belonging to one of these sets can reach any other permutation in this set with a single move of block *b*. Consider one such set: *Pb*. *pi*, *pj* are permutations in *Pb* *( i\< j)*, such that *pi* can reach *pj* by sliding *b* to the right.

**Lemma 1:** All permutations in *Pb* are one distance from each other.  
**Proof**: Any permutation in *Pb* can reach any other permutation in a single step by sliding *b*.

**Lemma 2:** All permutations in *Pb* are split over a maximum of two consecutive batches.  
**Proof**: Assume that for permutations *p* and *q* in *Pb*, *p* is in batch *i* and *q* is in batch *j*, such that *j \> i+1*. **Lemma 1** implies that *p* can reach *q* in a single move. Thus, *q* must be in the batch after *i*. This contradicts our assumption.

**Lemma 3**: permutations belonging to *Pb* have a **strict order of reachability** by sliding *b* to the right, meaning that while sliding *b*, a permutation *pi* in *Pb* will touch *pi+1* in *P*b before any other permutation *pj* in *Pb* where *i* \< *j*.  
**Proof:** Consider a permutation *p0* with block *b* starting at position 0\. If we were to slide this block by a single position, we’d get *p1*, continuing to slide will yield us the sequence *p2, p3, …* and so on. As we can see, sliding block *b* *y* times from *px* yields us *px+y*. Thus, *Pb* has a strict order of reachability via a single slide of *b* to the right.

**Lemma 4:** For the set of permutations in *Pb* that are in *processing* state, a permutation *pi*, while sliding *b*, either stops upon touching the next *processing* permutation *pi+1* in the set, or stops upon touching a *processed* permutation *q*, or touches only new *unprocessed* permutations.  
**Proof**: step (9.a.iii.3) will ensure that either *pi* stops processing sliding of *b*, upon touching *pi+1*, as that is in a *processing* state, or upon touching a *processed* permutation *q*. The only other state for all permutations touched is that they are in an *unprocessed* state.

**Lemma 5:** If permutation *p* hits a *processed* permutation *q* while sliding block *b*, then *p* is the only *processing* permutation that will hit *q* by sliding *b*.  
**Proof:** Assume that there exist two permutations *p* and *p’* in the *processing* state*,* that’d hit *q* by sliding *b*. But, since *p*, *p’* and *q* are in a **strictly ordered set**, according to **Lemma 3**, there is a strict ordering, i.e. one of the following orders must hold true: (*p, p’, q*), (*p, q, p’*), (*q, p, p’*), (*p’, p, q*), (*p’, q, p*), (*q, p’, p*).  
(*p, p’, q*) and  (*p’, p, q*), are the only orders where both *p* and *p’* might hit *q*, but in both of these scenarios, the first permutation will hit the second before hitting *q*, and step (9.a.iii.3) would prevent further sliding. Also, **Lemma 2** ensures that if *q* is in a *processed* state, then *p* and *p’* must be in the same batch together. Thus, our assumption is wrong.

**Lemma 6:** If a permutation *p* hits an *unprocessed* permutation *q* while sliding block *b* to right, then *p* is the only *processing* permutation that will hit *q* while sliding *b*.  
**Proof:** Assume that there exist two permutations *p* and *p’* in the *processing* state*,* that’d hit *q* by sliding *b*. But, since *p*, *p’* and *q* are in a **strictly ordered set**, according to **Lemma 3**, there is a strict ordering, i.e. one of the following orders must hold true: (*p, p’, q*), (*p, q, p’*), (*q, p, p’*), (*p’, p, q*), (*p’, q, p*), (*q, p’, p*).   
(*p, p’, q*) and  (*p’, p, q*), are the only orders where both *p* and *p’* might hit *q*, but in both of these scenarios, the first permutation will hit the second before hitting *q*, and step (9.a.iii.3) would prevent further sliding. Also, **Lemma 2** ensures that if *q* is in an unprocessed state, then *p* and *p’* must be in the same batch together. Thus, our assumption is wrong. This is the **same proof** as Lemma 5\.

**Lemma 7:** If a permutation *p* hits a *processing* permutation *q* while sliding block *b* to right, then *p* is the only *processing* permutation that will hit *q*.  
**Proof**: According to **Lemma 3**, there is a strict order of reachability in *Pb*. For the subset of *Pb*, let’s say *Pb’* which is in *processing* state, the strict order is also maintained. This implies that *pi* will reach *pi+1* in *Pb’* before reaching *pj* (*j \> i+1*). Due to this, step 9.a.iii.3 will ensure that each *pi* is stopped by *pi+1* in *Pb’*, and that *pi-1* is the only permutation in *Pb’* that can hit *pi*.

**Lemma 6** proves that **1** can only happen once for every *p*.  
**Lemma 4** proves that either **3a** happens at or **3b** happens, but never both.  
**Lemma 7** proves that if **3a** happens, it happens only once for every *p*.  
**Lemma 5** proves that if **3b** happens, it happens only once for every *p*.

Thus, the innermost loop has a time complexity of O(n\!\*n2) as there are *n\!\*n2* unique permutation and block combinations, and asymptotically, the innermost loop hits them at max *O(1)* times.

The marking of *processing*, *processed* and identifying the next batch, also asymptotically only takes *O(n\!*) time over all batches.

The space complexity of tracking the *visited* permutation block pair is *O(n\!\*n2*).

This gives an overall time complexity of O(n\!\*n2) and a space complexity of O(n\!\*n2).

# Parallelized Algorithm

1. Dn= set of distances of permutations in Sn,  
    such that i \< n\! Dn\[i\]= distance estimate of permutation *p* represented by label *i*  
2. Initialize distances and visited  
   1. **parallel for all** i, *0\<=p\<=n\!*,   
      1. set *Dn\[p\] \= 0*, initialize all distance estimates to *n*  
      2. **parallel for** block *\[i,j\]* in *p* : 0ij\<(n-1)  
         1. visited\[*p*\]\[*i*\]\[*j*\] \= false  
3. Set Dn\[0\]=0, distance of In is 0  
4. B= set of *unprocessed* labels with minimum distance estimates  
5. B={0}, start with batch of only *In*, represented by label 0  
6. Let *dcurr* \= 0, distance if *In* is 0  
7. **parallely for** **all** *p* in B, mark *p* as *processing*  
8. Let Bnext={}  
9. **parallel for** *p* in *B*  
   1. **parallel for** block *\[i,j\]* in *p* : 0ij\<(n-1)  
      1. size=(j-i+1)  // size of the block we are sliding  
      2. p'=p                      // The permutation reached by sliding block  
      3. for *k* : j\<k\<n  
         1. shift=(k-j)  // By how much we have slid the block  
         2. p'=p'+delta(p',k,k-size)  
         3. if *p’* is processed or processing, continue to next block in *9a*  
         4. if visited\[*p’*\]\[*i+shift*\]\[*j+shift*\]: continue to next block in (9.a)  
         5. visited\[*p’*\]\[*i+shift*\]\[*j+shift*\] \= true  
         6. if *acquire\_lock(*Dn\[p'\]*)* and Dn\[p'\]\>dcurr+1,   
            1. set Dn\[p'\]=dcurr+1   
            2. add *p’* to Bnext  
            3. *release\_lock(*Dn\[p'\]*)*  
         7. else, continue to next block in (9.a)  
10. **parallely for** **all** *p* in B, mark *p* as *processed*  
11. If Bnext is not empty,   
    1. set B=Bnext and *dcurr* \= *dcurr* \+ 1  
    2.  go to *7*  
12. else, end algorithm, Dnhas the distance of all labels

### Parallelized Complexity

**\[Work In Progress, need to accurately prove all the following, and also consider precomputation parallelization\!\]**

#### work (*T1*)

work for the parallel algorithm is the same as that for the linear algorithm at *O(n\!\*n2)*

#### **span (*T∞*)**

Span for the parallel algorithm is *O(b)* where b is the number of batches.  
**Proof:** 

Steps 1 to 6 run once, with step 2 having a span of *O(1)*. Step 7 has a span of *O(1)*.

Step 9 is parallelizable down to the innermost loop running on a single thread, meaning that the span of step 9 is also *O(1)*. Step 9.a.iii.5 also only occurs once for every permutation *p* and block *b* according to **Lemma 6**, and in the same batch, no two permutations will be performing 9.a.iii.4 and 9.a.iii.5 for the same *p, b* in the same batch, thus there’s no read-write race condition.

Step 10 is also run in parallel and has a span of *O(1)*. Step 11 and 12 have a complexity of *O(1)*.

Steps 7 to 11 run *O(b)* times, giving the overall span to be *O(b)*.

However, the precomputation might add to the span.

***Tp***:  
**speedup:**  
**parallelism:**

**Parallel Complexity of Precomputation**

**Label Generator**: Each permutation can be mapped to it’s label in O(n) time. This can be done in parallel for all n\! permutation, giving us:

work : O(n\!\*n)  
span: O(n)

**rs\_pre**: rs\_pre can also be run for every permutation parallely, giving us:

work: O(n\!\*n\*b(n))  
span: O(n\*b(n)), for practical purposes, O(n)

**mask\_pre**: Each permutation can again be processed in parallel, giving us:

work: O(n\!\*n\*b(n))  
span: O(n\*b(n)), practically, O(n)

Thus, including the precomputation, the overall complexity becomes:

work: O(n\!\*n2)  
span: O(n)  

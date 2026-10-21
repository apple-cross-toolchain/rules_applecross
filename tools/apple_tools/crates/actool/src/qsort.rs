//! macOS's `qsort`, whose order among equal elements is part of what Xcode
//! writes (docs/clean-room/actool.md). Translated from FreeBSD's
//! `stdlib/qsort.c` as Apple's libc ships it:
//!
//! Copyright (c) 1992, 1993
//!     The Regents of the University of California.  All rights reserved.
//!
//! Redistribution and use in source and binary forms, with or without
//! modification, are permitted provided that the following conditions
//! are met:
//! 1. Redistributions of source code must retain the above copyright
//!    notice, this list of conditions and the following disclaimer.
//! 2. Redistributions in binary form must reproduce the above copyright
//!    notice, this list of conditions and the following disclaimer in the
//!    documentation and/or other materials provided with the distribution.
//! 3. Neither the name of the University nor the names of its contributors
//!    may be used to endorse or promote products derived from this software
//!    without specific prior written permission.
//!
//! THIS SOFTWARE IS PROVIDED BY THE REGENTS AND CONTRIBUTORS ``AS IS'' AND
//! ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
//! IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
//! ARE DISCLAIMED.  IN NO EVENT SHALL THE REGENTS OR CONTRIBUTORS BE LIABLE
//! FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
//! DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
//! OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
//! HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
//! LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
//! OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
//! SUCH DAMAGE.

use std::cmp::Ordering;

fn isort<T>(a: &mut [T], cmp: &impl Fn(&T, &T) -> Ordering, limit: usize) -> bool {
    let mut swaps = 0;
    for m in 1..a.len() {
        let mut l = m;
        while l > 0 && cmp(&a[l - 1], &a[l]) == Ordering::Greater {
            a.swap(l, l - 1);
            l -= 1;
            swaps += 1;
            if limit != 0 && swaps > limit {
                return false;
            }
        }
    }
    true
}

fn med3<T>(a: &[T], x: usize, y: usize, z: usize, cmp: &impl Fn(&T, &T) -> Ordering) -> usize {
    let lt = |i: usize, j: usize| cmp(&a[i], &a[j]) == Ordering::Less;
    let gt = |i: usize, j: usize| cmp(&a[i], &a[j]) == Ordering::Greater;
    if lt(x, y) {
        if lt(y, z) { y } else if lt(x, z) { z } else { x }
    } else if gt(y, z) {
        y
    } else if lt(x, z) {
        x
    } else {
        z
    }
}

/// BSD's heapsort, which `qsort` falls back to past its depth limit.
fn heapsort<T: Copy>(a: &mut [T], cmp: &impl Fn(&T, &T) -> Ordering) {
    let n = a.len();
    if n <= 1 {
        return;
    }
    // Elements are numbered from 1.
    let at = |i: usize| i - 1;
    let create = |a: &mut [T], mut par: usize, nmemb: usize| {
        while par * 2 <= nmemb {
            let mut child = par * 2;
            if child < nmemb && cmp(&a[at(child)], &a[at(child + 1)]) == Ordering::Less {
                child += 1;
            }
            if cmp(&a[at(child)], &a[at(par)]) != Ordering::Greater {
                break;
            }
            a.swap(at(par), at(child));
            par = child;
        }
    };
    for l in (1..=n / 2).rev() {
        create(a, l, n);
    }
    let mut nmemb = n;
    while nmemb > 1 {
        let k = a[at(nmemb)];
        a[at(nmemb)] = a[at(1)];
        nmemb -= 1;
        let mut par = 1;
        while par * 2 <= nmemb {
            let mut child = par * 2;
            if child < nmemb && cmp(&a[at(child)], &a[at(child + 1)]) == Ordering::Less {
                child += 1;
            }
            a[at(par)] = a[at(child)];
            par = child;
        }
        loop {
            let child = par;
            par = child / 2;
            if child == 1 || cmp(&k, &a[at(par)]) == Ordering::Less {
                a[at(child)] = k;
                break;
            }
            a[at(child)] = a[at(par)];
        }
    }
}

fn sort<T: Copy>(mut a: &mut [T], cmp: &impl Fn(&T, &T) -> Ordering, mut depth: i32) {
    loop {
        let n = a.len();
        let exhausted = depth <= 0;
        depth -= 1;
        if exhausted {
            heapsort(a, cmp);
            return;
        }
        if n <= 7 {
            isort(a, cmp, 0);
            return;
        }
        let (mut pl, mut pm, mut pn) = (0, n / 2, n - 1);
        if n > 40 {
            let d = n / 8;
            pl = med3(a, pl, pl + d, pl + 2 * d, cmp);
            pm = med3(a, pm - d, pm, pm + d, cmp);
            pn = med3(a, pn - 2 * d, pn - d, pn, cmp);
        }
        pm = med3(a, pl, pm, pn, cmp);
        a.swap(0, pm);
        let mut swapped = false;
        let (mut pa, mut pb) = (1, 1);
        let (mut pc, mut pd) = (n - 1, n - 1);
        loop {
            while pb <= pc {
                match cmp(&a[pb], &a[0]) {
                    Ordering::Greater => break,
                    Ordering::Equal => {
                        swapped = true;
                        a.swap(pa, pb);
                        pa += 1;
                    }
                    Ordering::Less => {}
                }
                pb += 1;
            }
            while pb <= pc {
                match cmp(&a[pc], &a[0]) {
                    Ordering::Less => break,
                    Ordering::Equal => {
                        swapped = true;
                        a.swap(pc, pd);
                        pd -= 1;
                    }
                    Ordering::Greater => {}
                }
                pc -= 1;
            }
            if pb > pc {
                break;
            }
            a.swap(pb, pc);
            swapped = true;
            pb += 1;
            pc -= 1;
        }
        let d1 = std::cmp::min(pa, pb - pa);
        for t in 0..d1 {
            a.swap(t, pb - d1 + t);
        }
        let d1 = std::cmp::min(pd - pc, n - pd - 1);
        for t in 0..d1 {
            a.swap(pb + t, n - d1 + t);
        }
        // With nothing swapped, an insertion sort bounded to n/4 + 1 swaps,
        // whose swaps stand if it gives up.
        if !swapped && isort(a, cmp, 1 + n / 4) {
            return;
        }
        let (d1, d2) = (pb - pa, pd - pc);
        if d1 <= d2 {
            if d1 > 1 {
                sort(&mut a[..d1], cmp, depth);
            }
            if d2 <= 1 {
                return;
            }
            a = &mut a[n - d2..];
        } else {
            if d2 > 1 {
                sort(&mut a[n - d2..], cmp, depth);
            }
            if d1 <= 1 {
                return;
            }
            a = &mut a[..d1];
        }
    }
}

/// Sorts `a` as macOS's `qsort(3)` does.
pub fn qsort<T: Copy>(a: &mut [T], cmp: impl Fn(&T, &T) -> Ordering) {
    let depth = if a.is_empty() { 0 } else { 2 * (usize::BITS - 1 - a.len().leading_zeros()) as i32 };
    sort(a, &cmp, depth);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Orders of equal elements observed from macOS's qsort.
    #[test]
    fn equal_keys() {
        for (n, want) in [(7, vec![0, 1, 2, 3, 4, 5, 6]), (8, vec![7, 1, 2, 3, 4, 5, 6, 0]), (12, vec![11, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0])] {
            let mut a: Vec<usize> = (0..n).collect();
            qsort(&mut a, |_, _| Ordering::Equal);
            assert_eq!(a, want);
        }
    }

    /// One of the cases where macOS's qsort keeps neither the input order nor
    /// a stable one among equal keys (keys, then the resulting order).
    #[test]
    fn mixed_keys() {
        let keys = [14, 3, 10, 6, 11, 3, 9, 10, 5, 12];
        let mut a: Vec<usize> = (0..keys.len()).collect();
        qsort(&mut a, |x, y| keys[*x].cmp(&keys[*y]));
        assert_eq!(a, [1, 5, 8, 3, 6, 7, 2, 4, 9, 0]);
    }
}

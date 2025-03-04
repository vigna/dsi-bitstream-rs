/*
 * SPDX-FileCopyrightText: 2025 Tommaso Fontana
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/// Iters the points where the given function change value.
/// This only works for monotonic non decreasing functions.
///
/// Each call to next returns a tuple with the first input where the function
/// changes value and the new value.
///
/// This is useful to generate data following the implied distribution of a code.
pub struct FindChangePoints<F: Fn(u64) -> usize> {
    func: F,
    current: u64,
    prev_value: usize,
}

impl<F: Fn(u64) -> usize> FindChangePoints<F> {
    pub fn new(func: F) -> Self {
        Self {
            func,
            current: 0,
            prev_value: usize::MAX,
        }
    }
}

impl<F: Fn(u64) -> usize> Iterator for FindChangePoints<F> {
    /// (first input, output)
    type Item = (u64, usize);

    fn next(&mut self) -> Option<Self::Item> {
        // handle the first case, we don't need to search for the first change
        if self.current == 0 && self.prev_value == usize::MAX {
            self.prev_value = (self.func)(0);
            return Some((0, self.prev_value));
        }

        // Exponential search to find next potential change point starting from
        // the last change point.
        let mut step = 1;
        loop {
            // Avoid overflow, use <= instead of < because none of our codes
            // can encode u64::MAX, so let's just ignore it
            if u64::MAX - self.current <= step {
                return None;
            }
            // check if we found a change point
            let new_val = (self.func)(self.current + step);
            debug_assert!(
                new_val >= self.prev_value,
                "Function is not monotonic as f({}) = {} < {} = f({})",
                self.current + step,
                new_val,
                self.prev_value,
                self.current,
            );
            if new_val != self.prev_value {
                break;
            }
            step *= 2;
        }

        // Binary search in the last exponential step to find exact change point
        let mut left = self.current + step / 2;
        let mut right = self.current + step;

        while left < right {
            let mid = left + (right - left) / 2;
            let mid_val = (self.func)(mid);
            debug_assert!(
                mid_val >= self.prev_value,
                "Function is not monotonic as f({}) = {} < {} = f({})",
                mid,
                mid_val,
                self.prev_value,
                self.current,
            );
            if mid_val == self.prev_value {
                left = mid + 1;
            } else {
                right = mid;
            }
        }

        // Update state
        let new_value = (self.func)(left);
        debug_assert!(
            new_value >= self.prev_value,
            "Function is not monotonic as f({}) = {} < {} = f({})",
            left,
            new_value,
            self.prev_value,
            self.current,
        );

        self.current = left;
        self.prev_value = new_value;
        Some((self.current, new_value))
    }
}

#[cfg(test)]
mod test {
    use super::FindChangePoints;

    #[test]
    fn test_find_change_points() {
        test_func(crate::codes::len_gamma);
        test_func(crate::codes::len_delta);
        test_func(crate::codes::len_omega);
        test_func(|x| crate::codes::len_zeta(x, 3));
        test_func(|x| crate::codes::len_pi(x, 3));
    }

    fn test_func(func: impl Fn(u64) -> usize) {
        for (first, len) in FindChangePoints::new(&func) {
            // first check that the len is actually correct
            assert_eq!(func(first), len);
            // then check that it's the first one with that len
            if first > 0 {
                assert_ne!(func(first - 1), len);
            }
        }
    }
}

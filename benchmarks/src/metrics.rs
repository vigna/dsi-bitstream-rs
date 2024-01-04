/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/*!

Simple statistics module providing average, empirical standard deviation and quartiles.

*/

/// Structure to compute statistics from a stream.
pub struct MetricsStream {
    pub values: Vec<f64>,
    pub avg: f64,
    pub m2: f64,
}

/// The result of [`MetricsStream::finalize`].
#[derive(Debug)]
pub struct Metrics {
    pub percentile_75: f64,
    pub median: f64,
    pub percentile_25: f64,
    pub avg: f64,
    pub std: f64,
}

impl MetricsStream {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
            avg: 0.0,
            m2: 0.0,
        }
    }

    /// Ingest a value from the stream
    pub fn update(&mut self, value: f64) {
        assert!(value.is_finite());
        self.values.push(value);

        // Welford algorithm
        // https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance
        let delta = value - self.avg;
        self.avg += delta / self.values.len() as f64;
        let delta2 = value - self.avg;
        self.m2 += delta * delta2;
    }

    /// Consume this builder to get the statistics
    pub fn finalize(mut self) -> Metrics {
        let avg = if self.values.is_empty() {
            f64::NAN
        } else {
            self.avg
        };

        let std = if self.values.len() < 2 {
            f64::NAN
        } else {
            // Empirical standard deviation
            (self.m2 / (self.values.len() - 1) as f64).sqrt()
        };

        if self.values.len() < 2 {
            panic!(
                "Got {} values which is not enough for an std",
                self.values.len()
            );
        }
        self.values.sort_unstable_by(|a, b| a.total_cmp(b));

        let side = (self.values.len() - 1) / 2;
        Metrics {
            median: self.values[side],
            percentile_25: self.values[side / 2],
            percentile_75: self.values[side * 3 / 2],
            avg,
            std,
        }
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    DotProduct,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorHit {
    pub id: String,
    pub score: f32,
}

/// Compute cosine similarity using 4-way unrolled loops optimized for autovectorization (AVX2/AVX-512/NEON).
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vector dimensions must match");
    let len = a.len();

    let mut dot0 = 0.0f32;
    let mut dot1 = 0.0f32;
    let mut dot2 = 0.0f32;
    let mut dot3 = 0.0f32;

    let mut norm_a0 = 0.0f32;
    let mut norm_a1 = 0.0f32;
    let mut norm_a2 = 0.0f32;
    let mut norm_a3 = 0.0f32;

    let mut norm_b0 = 0.0f32;
    let mut norm_b1 = 0.0f32;
    let mut norm_b2 = 0.0f32;
    let mut norm_b3 = 0.0f32;

    let chunks = len / 4;
    for i in 0..chunks {
        let base = i * 4;
        let x0 = a[base];
        let x1 = a[base + 1];
        let x2 = a[base + 2];
        let x3 = a[base + 3];

        let y0 = b[base];
        let y1 = b[base + 1];
        let y2 = b[base + 2];
        let y3 = b[base + 3];

        dot0 += x0 * y0;
        dot1 += x1 * y1;
        dot2 += x2 * y2;
        dot3 += x3 * y3;

        norm_a0 += x0 * x0;
        norm_a1 += x1 * x1;
        norm_a2 += x2 * x2;
        norm_a3 += x3 * x3;

        norm_b0 += y0 * y0;
        norm_b1 += y1 * y1;
        norm_b2 += y2 * y2;
        norm_b3 += y3 * y3;
    }

    let mut dot = (dot0 + dot1) + (dot2 + dot3);
    let mut norm_a = (norm_a0 + norm_a1) + (norm_a2 + norm_a3);
    let mut norm_b = (norm_b0 + norm_b1) + (norm_b2 + norm_b3);

    // Tail elements
    for i in (chunks * 4)..len {
        let x = a[i];
        let y = b[i];
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a.sqrt() * norm_b.sqrt())
    }
}

/// Compute squared Euclidean ($L_2^2$) distance with unrolled SIMD loops.
pub fn euclidean_distance_squared(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vector dimensions must match");
    let len = a.len();
    let chunks = len / 4;

    let mut diff0 = 0.0f32;
    let mut diff1 = 0.0f32;
    let mut diff2 = 0.0f32;
    let mut diff3 = 0.0f32;

    for i in 0..chunks {
        let base = i * 4;
        let d0 = a[base] - b[base];
        let d1 = a[base + 1] - b[base + 1];
        let d2 = a[base + 2] - b[base + 2];
        let d3 = a[base + 3] - b[base + 3];

        diff0 += d0 * d0;
        diff1 += d1 * d1;
        diff2 += d2 * d2;
        diff3 += d3 * d3;
    }

    let mut sum = (diff0 + diff1) + (diff2 + diff3);
    for i in (chunks * 4)..len {
        let d = a[i] - b[i];
        sum += d * d;
    }
    sum
}

/// Compute dot product with unrolled SIMD loops.
pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vector dimensions must match");
    let len = a.len();
    let chunks = len / 4;

    let mut d0 = 0.0f32;
    let mut d1 = 0.0f32;
    let mut d2 = 0.0f32;
    let mut d3 = 0.0f32;

    for i in 0..chunks {
        let base = i * 4;
        d0 += a[base] * b[base];
        d1 += a[base + 1] * b[base + 1];
        d2 += a[base + 2] * b[base + 2];
        d3 += a[base + 3] * b[base + 3];
    }

    let mut sum = (d0 + d1) + (d2 + d3);
    for i in (chunks * 4)..len {
        sum += a[i] * b[i];
    }
    sum
}

/// In-Memory Vector Index supporting nearest-neighbor search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorIndex {
    pub dimension: usize,
    pub metric: DistanceMetric,
    pub entries: Vec<(String, Vec<f32>)>,
}

impl VectorIndex {
    pub fn new(dimension: usize, metric: DistanceMetric) -> Self {
        Self {
            dimension,
            metric,
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, id: String, vector: Vec<f32>) {
        assert_eq!(vector.len(), self.dimension, "Dimension mismatch");
        self.entries.push((id, vector));
    }

    /// Search for top-K nearest neighbors.
    pub fn search(&self, query: &[f32], k: usize) -> Vec<VectorHit> {
        let mut scored: Vec<VectorHit> = self
            .entries
            .iter()
            .map(|(id, vec)| {
                let score = match self.metric {
                    DistanceMetric::Cosine => cosine_similarity(query, vec),
                    DistanceMetric::Euclidean => -euclidean_distance_squared(query, vec).sqrt(),
                    DistanceMetric::DotProduct => dot_product(query, vec),
                };
                VectorHit {
                    id: id.clone(),
                    score,
                }
            })
            .collect();

        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let v1 = vec![1.0, 0.0, 0.0, 1.0];
        let v2 = vec![1.0, 0.0, 0.0, 1.0];
        let v3 = vec![0.0, 1.0, 0.0, 0.0];
        let v4 = vec![-1.0, 0.0, 0.0, -1.0];

        assert!((cosine_similarity(&v1, &v2) - 1.0).abs() < 1e-6);
        assert!((cosine_similarity(&v1, &v3) - 0.0).abs() < 1e-6);
        assert!((cosine_similarity(&v1, &v4) - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0, 0.0];
        assert_eq!(euclidean_distance_squared(&a, &b), 25.0);
    }

    #[test]
    fn test_vector_index_top_k() {
        let mut idx = VectorIndex::new(4, DistanceMetric::Cosine);
        idx.insert("doc_1".into(), vec![1.0, 0.0, 0.0, 0.0]);
        idx.insert("doc_2".into(), vec![0.9, 0.1, 0.0, 0.0]);
        idx.insert("doc_3".into(), vec![0.0, 1.0, 0.0, 0.0]);

        let query = vec![1.0, 0.0, 0.0, 0.0];
        let hits = idx.search(&query, 2);

        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].id, "doc_1");
        assert!((hits[0].score - 1.0).abs() < 1e-6);
        assert_eq!(hits[1].id, "doc_2");
    }
}

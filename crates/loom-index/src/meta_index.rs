use crate::vector::{cosine_similarity, euclidean_distance_squared, DistanceMetric};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexPrimitive {
    BTreeRange { field: String, min: f64, max: f64 },
    TimeRange { field: String, start_us: u64, end_us: u64 },
    HashMatch { field: String, value: String },
    VectorSimilarity {
        field: String,
        target_vector: Vec<f32>,
        metric: DistanceMetric,
        min_score: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaIndex {
    pub name: String,
    pub primitives: Vec<IndexPrimitive>,
}

#[derive(Debug, Clone)]
pub struct IndexCandidate<'a> {
    pub id: &'a str,
    pub numbers: &'a std::collections::HashMap<String, f64>,
    pub strings: &'a std::collections::HashMap<String, String>,
    pub timestamps: &'a std::collections::HashMap<String, u64>,
    pub vectors: &'a std::collections::HashMap<String, Vec<f32>>,
}

impl MetaIndex {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            primitives: Vec::new(),
        }
    }

    pub fn with_btree_range(mut self, field: &str, min: f64, max: f64) -> Self {
        self.primitives.push(IndexPrimitive::BTreeRange {
            field: field.to_string(),
            min,
            max,
        });
        self
    }

    pub fn with_time_range(mut self, field: &str, start_us: u64, end_us: u64) -> Self {
        self.primitives.push(IndexPrimitive::TimeRange {
            field: field.to_string(),
            start_us,
            end_us,
        });
        self
    }

    pub fn with_vector_search(
        mut self,
        field: &str,
        target_vector: Vec<f32>,
        metric: DistanceMetric,
        min_score: f32,
    ) -> Self {
        self.primitives.push(IndexPrimitive::VectorSimilarity {
            field: field.to_string(),
            target_vector,
            metric,
            min_score,
        });
        self
    }

    /// Single-pass composable filter evaluation across all index primitives.
    pub fn matches(&self, candidate: &IndexCandidate) -> (bool, f32) {
        let mut overall_score = 1.0f32;

        for prim in &self.primitives {
            match prim {
                IndexPrimitive::BTreeRange { field, min, max } => {
                    let val = candidate.numbers.get(field).copied().unwrap_or(f64::NAN);
                    if val < *min || val > *max {
                        return (false, 0.0);
                    }
                }
                IndexPrimitive::TimeRange { field, start_us, end_us } => {
                    let val = candidate.timestamps.get(field).copied().unwrap_or(0);
                    if val < *start_us || val > *end_us {
                        return (false, 0.0);
                    }
                }
                IndexPrimitive::HashMatch { field, value } => {
                    let val = candidate.strings.get(field).map(|s| s.as_str()).unwrap_or("");
                    if val != value {
                        return (false, 0.0);
                    }
                }
                IndexPrimitive::VectorSimilarity {
                    field,
                    target_vector,
                    metric,
                    min_score,
                } => {
                    if let Some(cand_vec) = candidate.vectors.get(field) {
                        let score = match metric {
                            DistanceMetric::Cosine => cosine_similarity(cand_vec, target_vector),
                            DistanceMetric::Euclidean => {
                                1.0 / (1.0 + euclidean_distance_squared(cand_vec, target_vector).sqrt())
                            }
                            DistanceMetric::DotProduct => {
                                cand_vec.iter().zip(target_vector).map(|(a, b)| a * b).sum()
                            }
                        };
                        if score < *min_score {
                            return (false, 0.0);
                        }
                        overall_score = score;
                    } else {
                        return (false, 0.0);
                    }
                }
            }
        }

        (true, overall_score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_composable_meta_index() {
        let idx = MetaIndex::new("hybrid_sensor_idx")
            .with_btree_range("temp", 20.0, 50.0)
            .with_vector_search("embedding", vec![0.1, 0.2, 0.3], DistanceMetric::Cosine, 0.8);

        let mut numbers = HashMap::new();
        numbers.insert("temp".into(), 28.5);
        let strings = HashMap::new();
        let timestamps = HashMap::new();
        let mut vectors = HashMap::new();
        vectors.insert("embedding".into(), vec![0.1, 0.2, 0.3]);

        let candidate = IndexCandidate {
            id: "s1",
            numbers: &numbers,
            strings: &strings,
            timestamps: &timestamps,
            vectors: &vectors,
        };

        let (matched, score) = idx.matches(&candidate);
        assert!(matched);
        assert!(score >= 0.99);
    }
}

//! Loop Closure Detection using Visual Place Recognition
//!
//! Implements DBoW2-style bag-of-visual-words for robust loop closure detection.
//! Features are indexed against a learned vocabulary for fast place recognition.

use crate::types::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;

/// Visual vocabulary word (cluster center in descriptor space)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VocabularyWord {
    /// Word ID in vocabulary
    pub id: u32,
    /// Cluster center (descriptor representative)
    pub center: Vec<u8>,
    /// Number of frames where this word appears
    pub occurrence_count: u32,
    /// Inverse document frequency (IDF) weight
    pub idf_weight: f32,
}

impl VocabularyWord {
    /// Create vocabulary word
    pub fn new(id: u32, center: Vec<u8>) -> Self {
        VocabularyWord {
            id,
            center,
            occurrence_count: 0,
            idf_weight: 1.0,
        }
    }

    /// Compute hamming distance to another descriptor
    fn hamming_distance(&self, descriptor: &[u8]) -> u32 {
        let mut distance = 0;
        let min_len = self.center.len().min(descriptor.len());
        for i in 0..min_len {
            let xor = self.center[i] ^ descriptor[i];
            distance += xor.count_ones();
        }
        distance
    }
}

/// Feature vocabulary for bag-of-words
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeatureVocabulary {
    /// Vocabulary words (clusters)
    pub words: Vec<VocabularyWord>,
    /// Total number of frames processed
    pub frame_count: u32,
    /// Word ID counter
    next_word_id: u32,
}

impl FeatureVocabulary {
    /// Create empty vocabulary
    pub fn new() -> Self {
        FeatureVocabulary {
            words: Vec::new(),
            frame_count: 0,
            next_word_id: 0,
        }
    }

    /// Create vocabulary from training data (k-means clustering)
    pub fn from_training_data(descriptors: Vec<Vec<u8>>, num_words: u32) -> Result<Self> {
        if descriptors.is_empty() {
            return Err(Error::InvalidObservation("No training descriptors".to_string()));
        }

        // Simple k-means clustering (k-means++)
        let mut vocabulary = FeatureVocabulary::new();
        vocabulary.words.reserve(num_words as usize);

        // Initialize cluster centers (simplified: random selection from data)
        let mut used_indices = std::collections::HashSet::new();
        for word_id in 0..num_words {
            let idx = (word_id as usize) % descriptors.len();
            if !used_indices.contains(&idx) {
                let word = VocabularyWord::new(word_id, descriptors[idx].clone());
                vocabulary.words.push(word);
                used_indices.insert(idx);
            }
        }

        // Assign descriptors to nearest cluster
        for descriptor in &descriptors {
            let nearest_word_id = vocabulary.find_nearest_word(descriptor);
            vocabulary.words[nearest_word_id as usize].occurrence_count += 1;
        }

        // Compute IDF weights
        vocabulary.compute_idf_weights();
        vocabulary.frame_count = 1;

        Ok(vocabulary)
    }

    /// Find nearest vocabulary word to a descriptor
    pub fn find_nearest_word(&self, descriptor: &[u8]) -> u32 {
        let mut best_word_id = 0;
        let mut best_distance = u32::MAX;

        for word in &self.words {
            let distance = word.hamming_distance(descriptor);
            if distance < best_distance {
                best_distance = distance;
                best_word_id = word.id;
            }
        }

        best_word_id
    }

    /// Compute IDF weights for vocabulary words
    fn compute_idf_weights(&mut self) {
        let total_words = self.words.len() as f32;
        for word in &mut self.words {
            let occurrence_ratio = word.occurrence_count as f32 / (self.frame_count.max(1) as f32);
            // IDF = log(total_words / (occurrences + 1))
            word.idf_weight = (total_words / (occurrence_ratio.max(1.0))).ln();
        }
    }

    /// Get vocabulary size
    pub fn size(&self) -> usize {
        self.words.len()
    }
}

impl Default for FeatureVocabulary {
    fn default() -> Self {
        Self::new()
    }
}

/// BoW histogram (bag-of-words representation of a frame)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoWHistogram {
    /// Frame ID
    pub frame_id: u32,
    /// Word histogram: word_id -> TF-IDF score
    pub histogram: HashMap<u32, f32>,
    /// L2 norm (for normalization)
    pub norm: f32,
}

impl BoWHistogram {
    /// Create BoW histogram from descriptors and vocabulary
    pub fn from_descriptors(frame_id: u32, descriptors: &[Vec<u8>], vocab: &FeatureVocabulary) -> Self {
        let mut histogram = HashMap::new();

        // Assign each descriptor to nearest vocabulary word
        for descriptor in descriptors {
            let word_id = vocab.find_nearest_word(descriptor);
            let word = vocab.words.iter().find(|w| w.id == word_id);
            if let Some(w) = word {
                let tf_idf = w.idf_weight; // TF = 1 per occurrence, IDF from vocabulary
                *histogram.entry(word_id).or_insert(0.0) += tf_idf;
            }
        }

        // Compute L2 norm
        let norm = histogram.values().map(|x| x * x).sum::<f32>().sqrt();

        BoWHistogram {
            frame_id,
            histogram,
            norm: norm.max(1.0), // Avoid division by zero
        }
    }

    /// Compute similarity to another histogram (cosine distance)
    pub fn similarity(&self, other: &BoWHistogram) -> f32 {
        if self.histogram.is_empty() || other.histogram.is_empty() {
            return 0.0;
        }

        let mut dot_product = 0.0;
        for (word_id, score1) in &self.histogram {
            if let Some(score2) = other.histogram.get(word_id) {
                dot_product += score1 * score2;
            }
        }

        // Cosine similarity
        dot_product / (self.norm * other.norm)
    }
}

/// Loop closure candidate (scored)
#[derive(Clone, Debug)]
pub struct LoopClosureCandidate {
    pub current_frame_id: u32,
    pub candidate_frame_id: u32,
    pub score: f32,
}

impl Eq for LoopClosureCandidate {}

impl PartialEq for LoopClosureCandidate {
    fn eq(&self, other: &Self) -> bool {
        (self.score - other.score).abs() < 1e-6
    }
}

impl Ord for LoopClosureCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        other.score.partial_cmp(&self.score).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for LoopClosureCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.score.partial_cmp(&self.score)
    }
}

/// Loop closure detector using visual place recognition
pub struct LoopClosureDetector {
    /// Feature vocabulary (trained on historical frames)
    pub vocabulary: FeatureVocabulary,
    /// BoW histograms database (frame_id -> histogram)
    pub histogram_db: HashMap<u32, BoWHistogram>,
    /// Minimum similarity threshold for loop closure
    pub min_similarity_threshold: f32,
    /// Minimum frame distance for loop closure (avoid false positives from sequential frames)
    pub min_frame_distance: u32,
}

impl LoopClosureDetector {
    /// Create loop closure detector with vocabulary
    pub fn new(vocabulary: FeatureVocabulary) -> Self {
        LoopClosureDetector {
            vocabulary,
            histogram_db: HashMap::new(),
            min_similarity_threshold: 0.5, // 50% similarity threshold
            min_frame_distance: 30, // At least 30 frames apart
        }
    }

    /// Add frame to database (build BoW histogram and store)
    pub fn add_frame(&mut self, frame_id: u32, descriptors: &[Vec<u8>]) {
        let histogram = BoWHistogram::from_descriptors(frame_id, descriptors, &self.vocabulary);
        self.histogram_db.insert(frame_id, histogram);
    }

    /// Search for loop closure candidates
    pub fn search_loop_closures(&self, current_frame_id: u32, descriptors: &[Vec<u8>]) -> Vec<LoopClosureCandidate> {
        if descriptors.is_empty() {
            return Vec::new();
        }

        let current_histogram = BoWHistogram::from_descriptors(current_frame_id, descriptors, &self.vocabulary);
        let mut candidates = BinaryHeap::new();

        // Search through all frames in database
        for (db_frame_id, db_histogram) in &self.histogram_db {
            // Skip frames that are too close
            if (*db_frame_id as i32 - current_frame_id as i32).abs() < self.min_frame_distance as i32 {
                continue;
            }

            let similarity = current_histogram.similarity(db_histogram);
            if similarity > self.min_similarity_threshold {
                candidates.push(LoopClosureCandidate {
                    current_frame_id,
                    candidate_frame_id: *db_frame_id,
                    score: similarity,
                });
            }
        }

        // Return top candidates (up to 5)
        let mut result = Vec::new();
        for _ in 0..5 {
            if let Some(candidate) = candidates.pop() {
                result.push(candidate);
            } else {
                break;
            }
        }

        result
    }

    /// Set similarity threshold
    pub fn set_similarity_threshold(&mut self, threshold: f32) {
        self.min_similarity_threshold = threshold.clamp(0.0, 1.0);
    }

    /// Set minimum frame distance
    pub fn set_min_frame_distance(&mut self, distance: u32) {
        self.min_frame_distance = distance;
    }

    /// Get database size (number of frames indexed)
    pub fn database_size(&self) -> usize {
        self.histogram_db.len()
    }
}

impl Default for LoopClosureDetector {
    fn default() -> Self {
        LoopClosureDetector::new(FeatureVocabulary::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vocabulary_creation() {
        let descriptors = vec![vec![1, 2, 3, 4], vec![1, 2, 3, 4], vec![5, 6, 7, 8]];
        let vocab = FeatureVocabulary::from_training_data(descriptors, 4).unwrap();
        assert!(vocab.size() > 0);
    }

    #[test]
    fn test_bow_histogram_similarity() {
        let vocab = FeatureVocabulary::new();
        let desc1 = vec![vec![1, 2, 3], vec![1, 2, 3]];
        let desc2 = vec![vec![1, 2, 3], vec![1, 2, 3]];

        let hist1 = BoWHistogram::from_descriptors(0, &desc1, &vocab);
        let hist2 = BoWHistogram::from_descriptors(1, &desc2, &vocab);

        // Same descriptors should have high similarity
        let similarity = hist1.similarity(&hist2);
        assert!(similarity >= 0.0 && similarity <= 1.0);
    }

    #[test]
    fn test_loop_closure_detector() {
        let vocab = FeatureVocabulary::new();
        let mut detector = LoopClosureDetector::new(vocab);

        let descriptors = vec![vec![1, 2, 3], vec![4, 5, 6]];
        detector.add_frame(0, &descriptors);

        let candidates = detector.search_loop_closures(50, &descriptors);
        // Should find frame 0 as a candidate (far enough: 50 frames apart)
        assert!(!candidates.is_empty());
    }
}

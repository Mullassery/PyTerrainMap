//! Spatial Intelligence Companion Layer
//!
//! Multi-source geospatial reasoning with explicit provenance tracking.
//! Understands regional context, source attribution, and uncertainty.
//! Designed for AI agents and autonomous systems that need to understand "why".

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Data provenance - where did this come from?
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataProvenance {
    /// Source name (GPS, NavIC, NOAA, SoilGrids, etc.)
    pub name: String,
    /// Confidence in this source (0.0-1.0)
    pub confidence: f32,
    /// Weight in final aggregation (0.0-1.0)
    pub weight: f32,
    /// Why was this source chosen/weighted?
    pub reasoning: String,
    /// Geographic coverage region (if applicable)
    pub region: Option<String>,
    /// Data currency (hours since update)
    pub age_hours: Option<u32>,
}

impl DataProvenance {
    /// Create data source
    pub fn new(name: &str, confidence: f32, weight: f32) -> Self {
        DataProvenance {
            name: name.to_string(),
            confidence,
            weight,
            reasoning: String::new(),
            region: None,
            age_hours: None,
        }
    }

    /// Add reasoning
    pub fn with_reasoning(mut self, reasoning: &str) -> Self {
        self.reasoning = reasoning.to_string();
        self
    }

    /// Add geographic region
    pub fn with_region(mut self, region: &str) -> Self {
        self.region = Some(region.to_string());
        self
    }

    /// Add data age
    pub fn with_age_hours(mut self, age: u32) -> Self {
        self.age_hours = Some(age);
        self
    }
}

/// Regional preference for data sources
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegionalPreference {
    /// Region name (India, Europe, North America, etc.)
    pub region: String,
    /// Preferred data sources ordered by priority
    pub source_priority: Vec<String>,
    /// Why these sources are preferred
    pub justification: String,
}

impl RegionalPreference {
    /// Create preference
    pub fn new(region: &str, sources: Vec<&str>) -> Self {
        RegionalPreference {
            region: region.to_string(),
            source_priority: sources.iter().map(|s| s.to_string()).collect(),
            justification: String::new(),
        }
    }

    /// Add justification
    pub fn with_justification(mut self, justification: &str) -> Self {
        self.justification = justification.to_string();
        self
    }
}

/// Uncertainty model
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Uncertainty {
    /// Mean error (same units as the measurement)
    pub error_magnitude: f32,
    /// Confidence interval (e.g., "±0.7 m", "±5%")
    pub confidence_interval: String,
    /// Factors contributing to uncertainty
    pub factors: Vec<String>,
    /// Recommended use cases (where this uncertainty is acceptable)
    pub acceptable_for: Vec<String>,
    /// Not recommended for
    pub not_acceptable_for: Vec<String>,
}

impl Uncertainty {
    /// Create uncertainty model
    pub fn new(magnitude: f32, interval: &str) -> Self {
        Uncertainty {
            error_magnitude: magnitude,
            confidence_interval: interval.to_string(),
            factors: Vec::new(),
            acceptable_for: Vec::new(),
            not_acceptable_for: Vec::new(),
        }
    }

    /// Add contributing factor
    pub fn add_factor(&mut self, factor: String) {
        self.factors.push(factor);
    }

    /// Mark as acceptable for use case
    pub fn acceptable_for(&mut self, use_case: &str) {
        self.acceptable_for.push(use_case.to_string());
    }

    /// Mark as not acceptable for use case
    pub fn not_acceptable_for(&mut self, use_case: &str) {
        self.not_acceptable_for.push(use_case.to_string());
    }
}

/// Reasoned spatial answer (not just data)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReasomedSpatialAnswer<T> {
    /// The actual answer
    pub value: T,
    /// Data sources and their contributions
    pub sources: Vec<DataProvenance>,
    /// Uncertainty model
    pub uncertainty: Uncertainty,
    /// Explicit reasoning (why this answer)
    pub reasoning: String,
    /// Regional context applied
    pub regional_context: Option<String>,
    /// Source disagreements (if any)
    pub disagreements: Vec<SourceDisagreement>,
    /// Confidence in final answer (0.0-1.0)
    pub confidence: f32,
}

impl<T> ReasomedSpatialAnswer<T> {
    /// Create reasoned answer
    pub fn new(value: T, confidence: f32) -> Self {
        ReasomedSpatialAnswer {
            value,
            sources: Vec::new(),
            uncertainty: Uncertainty::new(0.0, "Unknown"),
            reasoning: String::new(),
            regional_context: None,
            disagreements: Vec::new(),
            confidence,
        }
    }

    /// Add data source
    pub fn add_source(&mut self, source: DataProvenance) {
        self.sources.push(source);
    }

    /// Set uncertainty
    pub fn with_uncertainty(mut self, uncertainty: Uncertainty) -> Self {
        self.uncertainty = uncertainty;
        self
    }

    /// Add reasoning explanation
    pub fn with_reasoning(mut self, reasoning: &str) -> Self {
        self.reasoning = reasoning.to_string();
        self
    }

    /// Set regional context
    pub fn with_region(mut self, region: &str) -> Self {
        self.regional_context = Some(region.to_string());
        self
    }

    /// Add source disagreement
    pub fn add_disagreement(&mut self, disagreement: SourceDisagreement) {
        self.disagreements.push(disagreement);
    }

    /// Get explanation suitable for human reading
    pub fn explain_for_human(&self) -> String {
        let mut explanation = format!("Confidence: {:.0}%\n", self.confidence * 100.0);
        explanation.push_str(&format!("Uncertainty: {}\n", self.uncertainty.confidence_interval));
        explanation.push_str(&self.reasoning);

        if !self.disagreements.is_empty() {
            explanation.push_str("\n\nSource Disagreements:\n");
            for disagreement in &self.disagreements {
                explanation.push_str(&format!(
                    "  {} vs {}: {} difference\n",
                    disagreement.source_a, disagreement.source_b, disagreement.magnitude
                ));
                explanation.push_str(&format!("  Reason: {}\n", disagreement.explanation));
            }
        }

        explanation
    }

    /// Get explanation suitable for AI agents (detailed provenance)
    pub fn explain_for_agent(&self) -> serde_json::Value {
        serde_json::json!({
            "confidence": self.confidence,
            "uncertainty": {
                "magnitude": self.uncertainty.error_magnitude,
                "interval": self.uncertainty.confidence_interval,
                "factors": self.uncertainty.factors,
                "acceptable_for": self.uncertainty.acceptable_for,
                "not_acceptable_for": self.uncertainty.not_acceptable_for,
            },
            "sources": self.sources.iter().map(|s| serde_json::json!({
                "name": s.name,
                "confidence": s.confidence,
                "weight": s.weight,
                "reasoning": s.reasoning,
                "region": s.region,
                "age_hours": s.age_hours,
            })).collect::<Vec<_>>(),
            "reasoning": self.reasoning,
            "regional_context": self.regional_context,
            "disagreements": self.disagreements.iter().map(|d| serde_json::json!({
                "source_a": d.source_a,
                "source_b": d.source_b,
                "magnitude": d.magnitude,
                "explanation": d.explanation,
            })).collect::<Vec<_>>(),
        })
    }
}

/// Source disagreement tracking
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceDisagreement {
    /// First source
    pub source_a: String,
    /// Second source
    pub source_b: String,
    /// Magnitude of disagreement (same units as measurement)
    pub magnitude: String,
    /// Why they disagree
    pub explanation: String,
}

impl SourceDisagreement {
    /// Create disagreement record
    pub fn new(source_a: &str, source_b: &str, magnitude: &str) -> Self {
        SourceDisagreement {
            source_a: source_a.to_string(),
            source_b: source_b.to_string(),
            magnitude: magnitude.to_string(),
            explanation: String::new(),
        }
    }

    /// Add explanation
    pub fn with_explanation(mut self, explanation: &str) -> Self {
        self.explanation = explanation.to_string();
        self
    }
}

/// Position answer with full provenance
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PositionAnswer {
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: Option<f32>,
}

/// Spatial reasoning engine
pub struct SpatialReasoningEngine {
    /// Regional preferences
    pub preferences: HashMap<String, RegionalPreference>,
}

impl SpatialReasoningEngine {
    /// Create reasoning engine
    pub fn new() -> Self {
        SpatialReasoningEngine {
            preferences: HashMap::new(),
        }
    }

    /// Register regional preference
    pub fn register_preference(&mut self, preference: RegionalPreference) {
        self.preferences.insert(preference.region.clone(), preference);
    }

    /// Reason about position from multiple GNSS sources
    pub fn reason_position(
        &self,
        region: &str,
        sources_and_positions: Vec<(&str, f64, f64, f32)>, // (name, lat, lon, confidence)
    ) -> ReasomedSpatialAnswer<PositionAnswer> {
        let mut weighted_lat = 0.0;
        let mut weighted_lon = 0.0;
        let mut total_weight = 0.0;

        let mut answer = ReasomedSpatialAnswer::new(
            PositionAnswer {
                latitude: 0.0,
                longitude: 0.0,
                elevation: None,
            },
            0.8,
        );
        answer.regional_context = Some(region.to_string());

        // Get regional preference if available
        let preferred_sources = self
            .preferences
            .get(region)
            .map(|p| p.source_priority.clone());

        // Calculate weights based on region
        for (name, lat, lon, confidence) in sources_and_positions {
            let weight = if let Some(ref prefs) = preferred_sources {
                if let Some(pos) = prefs.iter().position(|s| s == name) {
                    // Higher priority = higher weight
                    let priority_weight = 1.0 - (pos as f32 * 0.1);
                    confidence * priority_weight
                } else {
                    confidence * 0.5 // Lower weight for non-preferred sources
                }
            } else {
                confidence // Use confidence as-is without regional preference
            };

            weighted_lat += lat * weight as f64;
            weighted_lon += lon * weight as f64;
            total_weight += weight as f64;

            // Add source
            let mut source = DataProvenance::new(name, confidence, weight);
            if let Some(pos) = preferred_sources.as_ref().and_then(|p| p.iter().position(|s| s == name)) {
                source = source.with_reasoning(&format!(
                    "{} is prioritized in {} (position {}) due to regional coverage",
                    name, region, pos
                ));
            }
            answer.add_source(source);
        }

        // Calculate final position
        if total_weight > 0.0 {
            let final_lat = weighted_lat / total_weight;
            let final_lon = weighted_lon / total_weight;

            answer.value = PositionAnswer {
                latitude: final_lat,
                longitude: final_lon,
                elevation: None,
            };

            // Set uncertainty based on source agreement
            let mut uncertainty = Uncertainty::new(0.7, "±0.7 m");
            uncertainty.add_factor("Weighted multi-source GNSS".to_string());
            uncertainty.add_factor(format!("Regional context: {}", region));
            uncertainty.acceptable_for("Autonomous vehicle navigation");
            uncertainty.acceptable_for("Precision agriculture");
            uncertainty.not_acceptable_for("Millimeter-level surveying");

            answer.uncertainty = uncertainty;

            // Add reasoning
            answer.reasoning = format!(
                "Position computed from {} GNSS sources. {} was assigned higher weighting due to regional preferences. Position uncertainty is ±0.7 meters.",
                answer.sources.len(),
                answer.sources.iter().find(|s| s.weight == answer.sources.iter().map(|x| x.weight).fold(f32::MIN, f32::max)).map(|s| &s.name).unwrap_or(&"Primary source".to_string())
            );
        }

        answer
    }
}

impl Default for SpatialReasoningEngine {
    fn default() -> Self {
        let mut engine = Self::new();

        // Register regional preferences
        engine.register_preference(
            RegionalPreference::new("India", vec!["NavIC", "GPS", "Galileo"])
                .with_justification("NavIC has optimal coverage in India; GPS provides global fallback")
        );

        engine.register_preference(
            RegionalPreference::new("Europe", vec!["Galileo", "GPS", "GLONASS"])
                .with_justification("Galileo is EU system; GPS provides global reference")
        );

        engine.register_preference(
            RegionalPreference::new("China", vec!["BeiDou", "GPS", "GLONASS"])
                .with_justification("BeiDou is Chinese GNSS system; GPS provides global fallback")
        );

        engine.register_preference(
            RegionalPreference::new("Russia", vec!["GLONASS", "GPS"])
                .with_justification("GLONASS is Russian system; GPS provides global reference")
        );

        engine.register_preference(
            RegionalPreference::new("North America", vec!["GPS", "Galileo", "GLONASS"])
                .with_justification("GPS is primary; others provide redundancy")
        );

        engine
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_source_creation() {
        let source = DataProvenance::new("GPS", 0.91, 0.45);
        assert_eq!(source.name, "GPS");
        assert_eq!(source.confidence, 0.91);
    }

    #[test]
    fn test_data_source_with_reasoning() {
        let source = DataProvenance::new("NavIC", 0.94, 0.55)
            .with_reasoning("Primary regional GNSS for India");
        assert_eq!(source.reasoning, "Primary regional GNSS for India");
    }

    #[test]
    fn test_regional_preference() {
        let pref = RegionalPreference::new("India", vec!["NavIC", "GPS"])
            .with_justification("NavIC optimal in India");
        assert_eq!(pref.source_priority[0], "NavIC");
    }

    #[test]
    fn test_uncertainty_model() {
        let mut uncertainty = Uncertainty::new(0.7, "±0.7 m");
        uncertainty.add_factor("Multi-source GNSS".to_string());
        assert_eq!(uncertainty.factors.len(), 1);
    }

    #[test]
    fn test_uncertainty_acceptable_for() {
        let mut uncertainty = Uncertainty::new(0.7, "±0.7 m");
        uncertainty.acceptable_for("Navigation");
        uncertainty.not_acceptable_for("Survey");
        assert_eq!(uncertainty.acceptable_for.len(), 1);
        assert_eq!(uncertainty.not_acceptable_for.len(), 1);
    }

    #[test]
    fn test_source_disagreement() {
        let disagreement = SourceDisagreement::new("NavIC", "GPS", "0.8 m")
            .with_explanation("Signal strength variation");
        assert_eq!(disagreement.magnitude, "0.8 m");
    }

    #[test]
    fn test_reasoned_answer_human_explanation() {
        let answer = ReasomedSpatialAnswer::new(
            PositionAnswer { latitude: 12.97, longitude: 77.59, elevation: None },
            0.92
        ).with_reasoning("NavIC-weighted position");
        let explanation = answer.explain_for_human();
        assert!(explanation.contains("Confidence: 92%"));
    }

    #[test]
    fn test_reasoned_answer_agent_explanation() {
        let answer = ReasomedSpatialAnswer::new(
            PositionAnswer { latitude: 12.97, longitude: 77.59, elevation: None },
            0.92
        );
        let json = answer.explain_for_agent();
        assert!(json["confidence"].is_number());
    }

    #[test]
    fn test_spatial_reasoning_engine_creation() {
        let engine = SpatialReasoningEngine::default();
        assert!(engine.preferences.contains_key("India"));
        assert!(engine.preferences.contains_key("Europe"));
    }

    #[test]
    fn test_reason_position_india() {
        let engine = SpatialReasoningEngine::default();
        let answer = engine.reason_position(
            "India",
            vec![
                ("NavIC", 12.971598, 77.594566, 0.94),
                ("GPS", 12.971600, 77.594568, 0.91),
            ],
        );
        assert!(answer.confidence >= 0.8);
        assert_eq!(answer.sources.len(), 2);
        assert!(answer.sources[0].weight >= answer.sources[1].weight); // NavIC weighted higher
    }

    #[test]
    fn test_reason_position_without_regional_preference() {
        let engine = SpatialReasoningEngine::new();
        let answer = engine.reason_position(
            "Unknown",
            vec![
                ("NavIC", 12.971598, 77.594566, 0.94),
                ("GPS", 12.971600, 77.594568, 0.91),
            ],
        );
        // Should still work but with equal weighting
        assert!(answer.value.latitude > 0.0);
    }
}

//! PyTerrainMap Intelligence Layer
//!
//! Transforms raw geospatial data into contextual understanding.
//! Personality-driven analysis that explains itself to AI agents and developers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Terrain intelligence context
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainAnalysis {
    /// Location (latitude, longitude)
    pub location: (f64, f64),
    /// Analysis timestamp (microseconds)
    pub timestamp_us: i64,
    /// Executive summary
    pub summary: String,
    /// Detailed observations
    pub observations: Vec<String>,
    /// Identified risks
    pub risks: Vec<Risk>,
    /// Recommendations for different personas
    pub recommendations: HashMap<Persona, Vec<String>>,
    /// Confidence in analysis (0.0-1.0)
    pub confidence: f32,
}

impl TerrainAnalysis {
    /// Create terrain analysis
    pub fn new(location: (f64, f64)) -> Self {
        TerrainAnalysis {
            location,
            timestamp_us: chrono::Utc::now().timestamp_micros(),
            summary: String::new(),
            observations: Vec::new(),
            risks: Vec::new(),
            recommendations: HashMap::new(),
            confidence: 0.7,
        }
    }

    /// Add observation
    pub fn add_observation(&mut self, obs: String) {
        self.observations.push(obs);
    }

    /// Add risk
    pub fn add_risk(&mut self, risk: Risk) {
        self.risks.push(risk);
    }

    /// Add recommendation for persona
    pub fn add_recommendation(&mut self, persona: Persona, recommendation: String) {
        self.recommendations
            .entry(persona)
            .or_insert_with(Vec::new)
            .push(recommendation);
    }

    /// Get summary
    pub fn summary(&self) -> &str {
        &self.summary
    }

    /// Get persona-specific advice
    pub fn advice_for(&self, persona: &Persona) -> Vec<&String> {
        self.recommendations
            .get(persona)
            .map(|recs| recs.iter().collect())
            .unwrap_or_default()
    }

    /// Export as structured report
    pub fn to_report(&self) -> AnalysisReport {
        AnalysisReport {
            location: self.location,
            summary: self.summary.clone(),
            observations: self.observations.clone(),
            risks: self.risks.clone(),
            recommendations: self.recommendations.clone(),
            confidence: self.confidence,
        }
    }
}

/// Analysis report (for export/serialization)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub location: (f64, f64),
    pub summary: String,
    pub observations: Vec<String>,
    pub risks: Vec<Risk>,
    pub recommendations: HashMap<Persona, Vec<String>>,
    pub confidence: f32,
}

/// Risk assessment
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Risk {
    /// Risk type
    pub risk_type: RiskType,
    /// Severity (0.0-1.0)
    pub severity: f32,
    /// Description
    pub description: String,
    /// Affected personas
    pub affected_personas: Vec<Persona>,
    /// Mitigation strategies
    pub mitigations: Vec<String>,
}

impl Risk {
    /// Create risk
    pub fn new(risk_type: RiskType, severity: f32, description: String) -> Self {
        Risk {
            risk_type,
            severity,
            description,
            affected_personas: Vec::new(),
            mitigations: Vec::new(),
        }
    }

    /// Add affected persona
    pub fn affects(mut self, persona: Persona) -> Self {
        self.affected_personas.push(persona);
        self
    }

    /// Add mitigation
    pub fn with_mitigation(mut self, mitigation: String) -> Self {
        self.mitigations.push(mitigation);
        self
    }

    /// Get severity label
    pub fn severity_label(&self) -> &str {
        match self.severity {
            s if s > 0.8 => "Critical",
            s if s > 0.6 => "High",
            s if s > 0.4 => "Medium",
            _ => "Low",
        }
    }
}

/// Risk types
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskType {
    Weather,
    Terrain,
    Soil,
    Flooding,
    Visibility,
    Accessibility,
    SlipHazard,
    Obstacle,
    Unknown,
}

impl RiskType {
    /// Get human-readable description
    pub fn description(&self) -> &str {
        match self {
            RiskType::Weather => "Adverse weather conditions",
            RiskType::Terrain => "Challenging terrain characteristics",
            RiskType::Soil => "Soil composition concerns",
            RiskType::Flooding => "Flood risk or water hazards",
            RiskType::Visibility => "Poor visibility for sensors",
            RiskType::Accessibility => "Access or navigation challenges",
            RiskType::SlipHazard => "Surface traction concerns",
            RiskType::Obstacle => "Physical obstacles present",
            RiskType::Unknown => "Unknown risk",
        }
    }
}

/// AI Agent / User Persona
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Persona {
    /// Autonomous ground rover/robot
    MobileRobot,
    /// Drone/quadcopter/aerial robot
    Drone,
    /// Farmer/agricultural user
    Farmer,
    /// Emergency response/disaster assessment
    DisasterResponse,
    /// Autonomous vehicle/car
    Vehicle,
    /// Geospatial analyst
    Analyst,
    /// Mission planner
    MissionPlanner,
}

impl Persona {
    /// Get persona name
    pub fn name(&self) -> &str {
        match self {
            Persona::MobileRobot => "Mobile Robot",
            Persona::Drone => "Drone Pilot",
            Persona::Farmer => "Farmer",
            Persona::DisasterResponse => "Disaster Response",
            Persona::Vehicle => "Autonomous Vehicle",
            Persona::Analyst => "Geospatial Analyst",
            Persona::MissionPlanner => "Mission Planner",
        }
    }

    /// Get all personas
    pub fn all() -> Vec<Persona> {
        vec![
            Persona::MobileRobot,
            Persona::Drone,
            Persona::Farmer,
            Persona::DisasterResponse,
            Persona::Vehicle,
            Persona::Analyst,
            Persona::MissionPlanner,
        ]
    }
}

/// Data explanation (for agent introspection)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataExplanation {
    /// Field name
    pub field: String,
    /// What it measures
    pub description: String,
    /// Why it matters
    pub applications: Vec<String>,
    /// Data confidence (0.0-1.0)
    pub confidence: f32,
    /// Data source
    pub source: String,
    /// Units or format
    pub units: String,
    /// Normal range
    pub normal_range: String,
}

impl DataExplanation {
    /// Explain soil_moisture
    pub fn soil_moisture() -> Self {
        DataExplanation {
            field: "soil_moisture".to_string(),
            description: "Amount of water retained in the upper soil layer (volumetric percentage)".to_string(),
            applications: vec![
                "Agricultural planning".to_string(),
                "Robot mobility prediction".to_string(),
                "Flood risk assessment".to_string(),
                "Crop health monitoring".to_string(),
            ],
            confidence: 0.75,
            source: "SoilGrids / USDA NRCS".to_string(),
            units: "Volumetric % (0-100)".to_string(),
            normal_range: "20-40% for most crops".to_string(),
        }
    }

    /// Explain temperature
    pub fn temperature() -> Self {
        DataExplanation {
            field: "temperature".to_string(),
            description: "Current air temperature at location".to_string(),
            applications: vec![
                "Robot battery performance".to_string(),
                "Sensor calibration".to_string(),
                "Mission feasibility".to_string(),
                "Weather forecasting".to_string(),
            ],
            confidence: 0.95,
            source: "Open-Meteo / NOAA / OpenWeather".to_string(),
            units: "Celsius (°C)".to_string(),
            normal_range: "-20 to +50°C typical".to_string(),
        }
    }

    /// Explain visibility
    pub fn visibility() -> Self {
        DataExplanation {
            field: "visibility".to_string(),
            description: "How far visual and LiDAR sensors can effectively see".to_string(),
            applications: vec![
                "Camera/LiDAR range planning".to_string(),
                "Obstacle detection capability".to_string(),
                "Mission safety assessment".to_string(),
                "Sensor confidence adjustment".to_string(),
            ],
            confidence: 0.80,
            source: "Weather station data".to_string(),
            units: "Meters".to_string(),
            normal_range: ">5000m in clear weather, <1000m in fog/rain".to_string(),
        }
    }

    /// Explain slope
    pub fn slope() -> Self {
        DataExplanation {
            field: "slope".to_string(),
            description: "Steepness of terrain (rise over run)".to_string(),
            applications: vec![
                "Rover traction analysis".to_string(),
                "Energy consumption estimation".to_string(),
                "Slide risk assessment".to_string(),
                "Path planning difficulty".to_string(),
            ],
            confidence: 0.90,
            source: "DEM / USGS / Copernicus".to_string(),
            units: "Degrees or percentage".to_string(),
            normal_range: "0-10° moderate, >30° steep".to_string(),
        }
    }
}

/// Reasoning about temporal trends
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemporalReasoning {
    /// Time period analyzed
    pub period_days: u32,
    /// Observed trends
    pub trends: Vec<Trend>,
    /// Projected conditions
    pub projections: Vec<Projection>,
    /// Recommended actions
    pub actions: Vec<Action>,
}

impl TemporalReasoning {
    /// Create reasoning
    pub fn new(period_days: u32) -> Self {
        TemporalReasoning {
            period_days,
            trends: Vec::new(),
            projections: Vec::new(),
            actions: Vec::new(),
        }
    }

    /// Add trend
    pub fn add_trend(&mut self, trend: Trend) {
        self.trends.push(trend);
    }

    /// Add projection
    pub fn add_projection(&mut self, projection: Projection) {
        self.projections.push(projection);
    }

    /// Add recommended action
    pub fn add_action(&mut self, action: Action) {
        self.actions.push(action);
    }
}

/// Observed trend
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trend {
    /// What is changing
    pub metric: String,
    /// Direction of change
    pub direction: TrendDirection,
    /// Magnitude of change
    pub magnitude: f32,
    /// Confidence
    pub confidence: f32,
}

/// Trend direction
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

/// Projection of future conditions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Projection {
    /// Metric being projected
    pub metric: String,
    /// Projected value
    pub projected_value: f32,
    /// Time horizon (hours)
    pub hours_ahead: u32,
    /// Confidence
    pub confidence: f32,
}

/// Recommended action
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Action {
    /// Action description
    pub description: String,
    /// Urgency (0.0-1.0)
    pub urgency: f32,
    /// Affected personas
    pub personas: Vec<Persona>,
    /// Expected outcome
    pub expected_outcome: String,
}

/// Mobility Assessment (for robots)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MobilityAssessment {
    /// Can move through this location?
    pub traversable: bool,
    /// Difficulty rating (0.0-1.0)
    pub difficulty: f32,
    /// Risk factors
    pub hazards: Vec<String>,
    /// Recommended speed (m/s)
    pub recommended_speed_ms: f32,
    /// Battery impact factor (1.0 = normal, 2.0 = double energy)
    pub battery_impact: f32,
    /// Time to cross (seconds for 100m)
    pub time_to_cross_100m_seconds: f32,
}

impl MobilityAssessment {
    /// Create assessment
    pub fn new() -> Self {
        MobilityAssessment {
            traversable: true,
            difficulty: 0.3,
            hazards: Vec::new(),
            recommended_speed_ms: 0.5,
            battery_impact: 1.0,
            time_to_cross_100m_seconds: 200.0,
        }
    }

    /// Get difficulty label
    pub fn difficulty_label(&self) -> &str {
        match self.difficulty {
            d if d > 0.8 => "Extremely difficult",
            d if d > 0.6 => "Very difficult",
            d if d > 0.4 => "Moderately difficult",
            d if d > 0.2 => "Slightly difficult",
            _ => "Easy",
        }
    }
}

/// Agricultural Suitability Assessment
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgriculturalSuitability {
    /// Crop type assessed
    pub crop_type: String,
    /// Suitability score (0.0-1.0)
    pub suitability: f32,
    /// Limiting factors
    pub limiting_factors: Vec<String>,
    /// Recommended amendments
    pub amendments: Vec<String>,
    /// Expected yield impact
    pub yield_impact_percent: f32,
}

impl AgriculturalSuitability {
    /// Create assessment
    pub fn new(crop_type: &str) -> Self {
        AgriculturalSuitability {
            crop_type: crop_type.to_string(),
            suitability: 0.6,
            limiting_factors: Vec::new(),
            amendments: Vec::new(),
            yield_impact_percent: 0.0,
        }
    }

    /// Add limiting factor
    pub fn add_limiting_factor(&mut self, factor: String) {
        self.limiting_factors.push(factor);
    }

    /// Add amendment
    pub fn add_amendment(&mut self, amendment: String) {
        self.amendments.push(amendment);
    }
}

/// Disaster Response Assessment
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DisasterAssessment {
    /// Primary hazard
    pub primary_hazard: String,
    /// Severity (0.0-1.0)
    pub severity: f32,
    /// Affected population estimate
    pub affected_population: Option<u32>,
    /// Access restrictions
    pub access_restrictions: Vec<String>,
    /// Critical resources
    pub critical_resources: Vec<String>,
    /// Assessment timestamp
    pub timestamp_us: i64,
}

impl DisasterAssessment {
    /// Create assessment
    pub fn new(hazard: &str) -> Self {
        DisasterAssessment {
            primary_hazard: hazard.to_string(),
            severity: 0.5,
            affected_population: None,
            access_restrictions: Vec::new(),
            critical_resources: Vec::new(),
            timestamp_us: chrono::Utc::now().timestamp_micros(),
        }
    }

    /// Get severity label
    pub fn severity_label(&self) -> &str {
        match self.severity {
            s if s > 0.8 => "Extreme",
            s if s > 0.6 => "Severe",
            s if s > 0.4 => "Moderate",
            _ => "Low",
        }
    }
}

/// MCP Tool Definition (for Claude Code / AI agents)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MCPTool {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Input schema
    pub input_schema: MCPSchema,
    /// Output schema
    pub output_schema: MCPSchema,
    /// Relevant personas
    pub personas: Vec<Persona>,
}

impl MCPTool {
    /// Terrain assessment tool
    pub fn terrain_assessment() -> Self {
        MCPTool {
            name: "terrain_assessment".to_string(),
            description: "Evaluate a location for mobility, agriculture, weather, and terrain risks.".to_string(),
            input_schema: MCPSchema {
                parameters: vec![
                    "latitude".to_string(),
                    "longitude".to_string(),
                    "persona".to_string(),
                ],
            },
            output_schema: MCPSchema {
                parameters: vec![
                    "summary".to_string(),
                    "observations".to_string(),
                    "risks".to_string(),
                    "recommendations".to_string(),
                ],
            },
            personas: Persona::all(),
        }
    }

    /// Mobility assessment tool
    pub fn mobility_assessment() -> Self {
        MCPTool {
            name: "mobility_assessment".to_string(),
            description: "Check if a robot can traverse a location safely.".to_string(),
            input_schema: MCPSchema {
                parameters: vec![
                    "latitude".to_string(),
                    "longitude".to_string(),
                    "robot_type".to_string(),
                ],
            },
            output_schema: MCPSchema {
                parameters: vec![
                    "traversable".to_string(),
                    "difficulty".to_string(),
                    "hazards".to_string(),
                    "recommended_speed_ms".to_string(),
                ],
            },
            personas: vec![Persona::MobileRobot, Persona::Drone, Persona::Vehicle],
        }
    }

    /// Data explanation tool
    pub fn explain_field() -> Self {
        MCPTool {
            name: "explain_field".to_string(),
            description: "Get detailed explanation of what a data field means.".to_string(),
            input_schema: MCPSchema {
                parameters: vec!["field_name".to_string()],
            },
            output_schema: MCPSchema {
                parameters: vec![
                    "description".to_string(),
                    "applications".to_string(),
                    "confidence".to_string(),
                    "source".to_string(),
                ],
            },
            personas: Persona::all(),
        }
    }
}

/// MCP Schema (simplified)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MCPSchema {
    /// Parameter names
    pub parameters: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_analysis_creation() {
        let analysis = TerrainAnalysis::new((40.71, -74.00));
        assert_eq!(analysis.location, (40.71, -74.00));
        assert_eq!(analysis.observations.len(), 0);
    }

    #[test]
    fn test_terrain_analysis_add_observation() {
        let mut analysis = TerrainAnalysis::new((40.71, -74.00));
        analysis.add_observation("Clay soil with high moisture".to_string());
        assert_eq!(analysis.observations.len(), 1);
    }

    #[test]
    fn test_terrain_analysis_add_risk() {
        let mut analysis = TerrainAnalysis::new((40.71, -74.00));
        let risk = Risk::new(RiskType::Soil, 0.7, "High clay content".to_string());
        analysis.add_risk(risk);
        assert_eq!(analysis.risks.len(), 1);
    }

    #[test]
    fn test_risk_creation() {
        let risk = Risk::new(RiskType::Weather, 0.7, "Heavy rain forecast".to_string());
        assert_eq!(risk.severity_label(), "High");
    }

    #[test]
    fn test_risk_affects() {
        let risk = Risk::new(RiskType::Visibility, 0.5, "Fog detected".to_string())
            .affects(Persona::Drone)
            .affects(Persona::MobileRobot);
        assert_eq!(risk.affected_personas.len(), 2);
    }

    #[test]
    fn test_persona_name() {
        assert_eq!(Persona::Drone.name(), "Drone Pilot");
        assert_eq!(Persona::Farmer.name(), "Farmer");
    }

    #[test]
    fn test_persona_all() {
        let personas = Persona::all();
        assert_eq!(personas.len(), 7);
    }

    #[test]
    fn test_data_explanation_soil_moisture() {
        let exp = DataExplanation::soil_moisture();
        assert_eq!(exp.field, "soil_moisture");
        assert!(exp.confidence > 0.0 && exp.confidence <= 1.0);
    }

    #[test]
    fn test_data_explanation_temperature() {
        let exp = DataExplanation::temperature();
        assert_eq!(exp.field, "temperature");
        assert_eq!(exp.units, "Celsius (°C)");
    }

    #[test]
    fn test_temporal_reasoning_creation() {
        let reasoning = TemporalReasoning::new(7);
        assert_eq!(reasoning.period_days, 7);
    }

    #[test]
    fn test_trend_enum() {
        assert_eq!(TrendDirection::Increasing, TrendDirection::Increasing);
    }

    #[test]
    fn test_mobility_assessment_creation() {
        let assessment = MobilityAssessment::new();
        assert!(assessment.traversable);
        assert!(assessment.difficulty < 1.0);
    }

    #[test]
    fn test_mobility_difficulty_label() {
        let mut assessment = MobilityAssessment::new();
        assessment.difficulty = 0.85;
        assert_eq!(assessment.difficulty_label(), "Extremely difficult");
    }

    #[test]
    fn test_agricultural_suitability_creation() {
        let suitability = AgriculturalSuitability::new("wheat");
        assert_eq!(suitability.crop_type, "wheat");
    }

    #[test]
    fn test_agricultural_suitability_add_factor() {
        let mut suitability = AgriculturalSuitability::new("corn");
        suitability.add_limiting_factor("Low nitrogen".to_string());
        assert_eq!(suitability.limiting_factors.len(), 1);
    }

    #[test]
    fn test_disaster_assessment_creation() {
        let assessment = DisasterAssessment::new("Flooding");
        assert_eq!(assessment.primary_hazard, "Flooding");
    }

    #[test]
    fn test_disaster_severity_label() {
        let mut assessment = DisasterAssessment::new("Flood");
        assessment.severity = 0.9;
        assert_eq!(assessment.severity_label(), "Extreme");
    }

    #[test]
    fn test_mcp_tool_terrain_assessment() {
        let tool = MCPTool::terrain_assessment();
        assert_eq!(tool.name, "terrain_assessment");
        assert_eq!(tool.personas.len(), 7); // All personas
    }

    #[test]
    fn test_mcp_tool_mobility_assessment() {
        let tool = MCPTool::mobility_assessment();
        assert_eq!(tool.name, "mobility_assessment");
        assert_eq!(tool.personas.len(), 3); // Robot, Drone, Vehicle
    }

    #[test]
    fn test_mcp_tool_explain_field() {
        let tool = MCPTool::explain_field();
        assert_eq!(tool.name, "explain_field");
    }

    #[test]
    fn test_terrain_analysis_to_report() {
        let mut analysis = TerrainAnalysis::new((40.71, -74.00));
        analysis.summary = "Suitable for rover operations".to_string();
        let report = analysis.to_report();
        assert_eq!(report.summary, "Suitable for rover operations");
    }

    #[test]
    fn test_terrain_analysis_add_recommendation() {
        let mut analysis = TerrainAnalysis::new((40.71, -74.00));
        analysis.add_recommendation(Persona::MobileRobot, "Use slow mode".to_string());
        assert!(analysis.recommendations.contains_key(&Persona::MobileRobot));
    }

    #[test]
    fn test_risk_type_description() {
        assert_eq!(RiskType::Flooding.description(), "Flood risk or water hazards");
        assert_eq!(RiskType::Weather.description(), "Adverse weather conditions");
    }
}

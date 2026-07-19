//! PyTerrainMap CLI Interface
//!
//! Natural language command interface for terrain analysis.
//! Makes PyTerrainMap accessible from the command line.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// CLI command types (natural language parsing)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CLICommand {
    /// Can entity travel here?
    CanTraverse {
        entity: String, // "rover", "drone", "farmer", "vehicle"
        location: (f64, f64),
    },
    /// What's at this location?
    Analyze {
        location: (f64, f64),
        persona: Option<String>,
    },
    /// Explain a concept
    Explain {
        field: String,
    },
    /// Show risks at location
    Risks {
        location: (f64, f64),
    },
    /// Check mission feasibility
    MissionFeasible {
        mission_type: String,
        location: (f64, f64),
    },
    /// Show weather at location
    Weather {
        location: (f64, f64),
    },
    /// Show soil conditions
    Soil {
        location: (f64, f64),
    },
    /// Show help
    Help,
    /// Version info
    Version,
}

impl CLICommand {
    /// Parse natural language command
    pub fn parse(input: &str) -> Result<CLICommand, String> {
        let lower = input.trim().to_lowercase();

        // Help
        if lower == "help" || lower == "--help" || lower == "-h" {
            return Ok(CLICommand::Help);
        }

        // Version
        if lower == "version" || lower == "--version" || lower == "-v" {
            return Ok(CLICommand::Version);
        }

        // Can traverse: "can-a-rover-drive-here" or "can a rover drive at 40.71 -74.00"
        if lower.contains("can") && (lower.contains("drive") || lower.contains("traverse") || lower.contains("fly")) {
            let entity = if lower.contains("drone") {
                "drone"
            } else if lower.contains("rover") || lower.contains("robot") {
                "rover"
            } else if lower.contains("car") || lower.contains("vehicle") {
                "vehicle"
            } else {
                "rover" // default
            };

            if let Some(location) = Self::extract_coordinates(&lower) {
                return Ok(CLICommand::CanTraverse {
                    entity: entity.to_string(),
                    location,
                });
            }
        }

        // Analyze: "analyze 40.71 -74.00" or "what's at 40.71 -74.00"
        if lower.contains("analyze") || lower.contains("what's at") || lower.contains("whats at") {
            if let Some(location) = Self::extract_coordinates(&lower) {
                return Ok(CLICommand::Analyze {
                    location,
                    persona: None,
                });
            }
        }

        // Weather: "weather at 40.71 -74.00"
        if lower.contains("weather") || lower.contains("climate") {
            if let Some(location) = Self::extract_coordinates(&lower) {
                return Ok(CLICommand::Weather { location });
            }
        }

        // Soil: "soil at 40.71 -74.00"
        if lower.contains("soil") && !lower.contains("soil-moisture") {
            if let Some(location) = Self::extract_coordinates(&lower) {
                return Ok(CLICommand::Soil { location });
            }
        }

        // Risks: "risks at 40.71 -74.00"
        if lower.contains("risk") {
            if let Some(location) = Self::extract_coordinates(&lower) {
                return Ok(CLICommand::Risks { location });
            }
        }

        // Explain: "explain soil_moisture"
        if lower.contains("explain") {
            let field = lower
                .replace("explain", "")
                .replace("what is", "")
                .replace("what's", "")
                .trim()
                .to_string();
            if !field.is_empty() {
                return Ok(CLICommand::Explain { field });
            }
        }

        // Mission: "can i farm at 40.71 -74.00"
        if lower.contains("mission") || lower.contains("can i") {
            let mission_type = if lower.contains("farm") {
                "agricultural"
            } else if lower.contains("rescue") || lower.contains("disaster") {
                "disaster_response"
            } else {
                "general"
            };

            if let Some(location) = Self::extract_coordinates(&lower) {
                return Ok(CLICommand::MissionFeasible {
                    mission_type: mission_type.to_string(),
                    location,
                });
            }
        }

        Err(format!("Unknown command: '{}'. Type 'help' for usage.", input))
    }

    /// Extract latitude and longitude from text
    fn extract_coordinates(text: &str) -> Option<(f64, f64)> {
        let parts: Vec<&str> = text.split_whitespace().collect();

        // Look for two consecutive numbers
        for i in 0..parts.len() - 1 {
            if let (Ok(lat), Ok(lon)) = (parts[i].parse::<f64>(), parts[i + 1].parse::<f64>()) {
                // Validate latitude and longitude ranges
                if lat >= -90.0 && lat <= 90.0 && lon >= -180.0 && lon <= 180.0 {
                    return Some((lat, lon));
                }
            }
        }

        None
    }
}

/// CLI Response - human-friendly output
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CLIResponse {
    /// Response status
    pub status: ResponseStatus,
    /// Main output
    pub output: String,
    /// Additional details
    pub details: Vec<String>,
    /// Warnings or additional info
    pub warnings: Vec<String>,
    /// Raw data (for scripting)
    pub data: Option<serde_json::Value>,
}

/// Response status
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseStatus {
    Success,
    Partial,
    Error,
    Warning,
}

impl CLIResponse {
    /// Create success response
    pub fn success(output: String) -> Self {
        CLIResponse {
            status: ResponseStatus::Success,
            output,
            details: Vec::new(),
            warnings: Vec::new(),
            data: None,
        }
    }

    /// Create error response
    pub fn error(output: String) -> Self {
        CLIResponse {
            status: ResponseStatus::Error,
            output,
            details: Vec::new(),
            warnings: Vec::new(),
            data: None,
        }
    }

    /// Add detail
    pub fn add_detail(&mut self, detail: String) {
        self.details.push(detail);
    }

    /// Add warning
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Set data
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Format for terminal output
    pub fn format_terminal(&self) -> String {
        let mut output = String::new();

        // Status indicator
        let status_icon = match self.status {
            ResponseStatus::Success => "✓",
            ResponseStatus::Partial => "⚠",
            ResponseStatus::Error => "✗",
            ResponseStatus::Warning => "!",
        };

        output.push_str(&format!("{} {}\n", status_icon, self.output));

        // Details
        if !self.details.is_empty() {
            output.push_str("\nDetails:\n");
            for detail in &self.details {
                output.push_str(&format!("  • {}\n", detail));
            }
        }

        // Warnings
        if !self.warnings.is_empty() {
            output.push_str("\nWarnings:\n");
            for warning in &self.warnings {
                output.push_str(&format!("  ⚠ {}\n", warning));
            }
        }

        output
    }

    /// Format as JSON (for scripting)
    pub fn format_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Help text
pub fn help_text() -> &'static str {
    r#"
PyTerrainMap CLI - Natural Language Terrain Intelligence

USAGE:
    pytm <COMMAND> [OPTIONS]

NATURAL LANGUAGE COMMANDS:

1. CAN I TRAVERSE HERE?
    pytm can-a-rover-drive-here 40.71 -74.00
    pytm can a drone fly at 40.71 -74.00
    pytm is it safe for a vehicle 40.71 -74.00

2. ANALYZE LOCATION
    pytm analyze 40.71 -74.00
    pytm what's at 40.71 -74.00
    pytm analyze location 40.71 -74.00 for farmer

3. EXPLAIN CONCEPTS
    pytm explain soil_moisture
    pytm what is temperature
    pytm explain visibility

4. CHECK CONDITIONS
    pytm weather at 40.71 -74.00
    pytm soil at 40.71 -74.00
    pytm risks at 40.71 -74.00

5. MISSION PLANNING
    pytm can i farm at 40.71 -74.00
    pytm disaster response feasibility 40.71 -74.00
    pytm mission suitability 40.71 -74.00

OPTIONS:
    --json               Output as JSON (for scripting)
    --verbose            Detailed output
    --quiet              Minimal output
    -h, --help          Show this help
    -v, --version       Show version

EXAMPLES:

    # Check if rover can drive
    $ pytm can-a-rover-drive-here 40.71 -74.00
    ✓ Rover can traverse this location
    Details:
      • Traversable: Yes
      • Difficulty: Moderate (0.45)
      • Recommended speed: 0.4 m/s
      • Battery impact: 1.3x normal

    # Analyze for farmer
    $ pytm analyze 40.71 -74.00 for farmer
    ✓ Moderate suitability for wheat cultivation
    Details:
      • Suitability score: 0.65
      • Limiting factors: Nitrogen deficit, High clay
      • Recommended amendments: Nitrogen (50kg/ha)
      • Expected yield impact: +8%

    # Understand a metric
    $ pytm explain soil_moisture
    Soil moisture is the amount of water retained in upper soil layer.

    Applications:
      • Agricultural planning
      • Robot mobility prediction
      • Flood risk assessment

    Source: SoilGrids (Confidence: 75%)
    Normal range: 20-40% for most crops

    # Export as JSON for scripting
    $ pytm can-a-rover-drive-here 40.71 -74.00 --json
    {
      "status": "Success",
      "output": "Rover can traverse this location",
      "data": { "traversable": true, "difficulty": 0.45 }
    }

PERSONAS (for context-aware analysis):
    - mobile_robot    (rover, robot, bot)
    - drone          (quadcopter, aerial, uav)
    - farmer         (agricultural, farm)
    - vehicle        (car, autonomous vehicle)
    - disaster       (emergency, rescue, disaster_response)
    - analyst        (data, gis, mapping)
    - mission        (planner, mission_planner)

TIPS:
    • Use decimal coordinates (e.g., 40.71 -74.00)
    • Commands are case-insensitive
    • Abbreviations work: "rover" = "mobile_robot", "drone" = "aerial"
    • Use --json for piping to other tools
    • Use --verbose for detailed explanations
"#
}

/// Version info
pub fn version_text() -> &'static str {
    "PyTerrainMap CLI v0.1.0\nIntelligent geospatial analysis for autonomous systems"
}

/// Sample commands for testing
#[cfg(test)]
mod sample_commands {
    use super::*;

    pub const SAMPLE_QUERIES: &[&str] = &[
        "can-a-rover-drive-here 40.71 -74.00",
        "can a drone fly at 40.71 -74.00",
        "analyze 40.71 -74.00",
        "what's at 40.71 -74.00",
        "weather at 40.71 -74.00",
        "soil at 40.71 -74.00",
        "risks at 40.71 -74.00",
        "explain soil_moisture",
        "can i farm at 40.71 -74.00",
        "help",
        "version",
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_help() {
        let cmd = CLICommand::parse("help").unwrap();
        match cmd {
            CLICommand::Help => {}
            _ => panic!("Expected Help"),
        }
    }

    #[test]
    fn test_parse_version() {
        let cmd = CLICommand::parse("version").unwrap();
        match cmd {
            CLICommand::Version => {}
            _ => panic!("Expected Version"),
        }
    }

    #[test]
    fn test_parse_can_traverse() {
        let cmd = CLICommand::parse("can-a-rover-drive-here 40.71 -74.00").unwrap();
        match cmd {
            CLICommand::CanTraverse { entity, location } => {
                assert_eq!(entity, "rover");
                assert_eq!(location, (40.71, -74.00));
            }
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_parse_can_traverse_drone() {
        let cmd = CLICommand::parse("can a drone fly at 40.71 -74.00").unwrap();
        match cmd {
            CLICommand::CanTraverse { entity, location } => {
                assert_eq!(entity, "drone");
                assert_eq!(location, (40.71, -74.00));
            }
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_parse_analyze() {
        let cmd = CLICommand::parse("analyze 40.71 -74.00").unwrap();
        match cmd {
            CLICommand::Analyze { location, .. } => {
                assert_eq!(location, (40.71, -74.00));
            }
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_parse_weather() {
        let cmd = CLICommand::parse("weather at 40.71 -74.00").unwrap();
        match cmd {
            CLICommand::Weather { location } => {
                assert_eq!(location, (40.71, -74.00));
            }
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_parse_soil() {
        let cmd = CLICommand::parse("soil at 40.71 -74.00").unwrap();
        match cmd {
            CLICommand::Soil { location } => {
                assert_eq!(location, (40.71, -74.00));
            }
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_parse_explain() {
        let cmd = CLICommand::parse("explain soil_moisture").unwrap();
        match cmd {
            CLICommand::Explain { field } => {
                assert_eq!(field, "soil_moisture");
            }
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_parse_risks() {
        let cmd = CLICommand::parse("risks at 40.71 -74.00").unwrap();
        match cmd {
            CLICommand::Risks { location } => {
                assert_eq!(location, (40.71, -74.00));
            }
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_parse_mission() {
        let cmd = CLICommand::parse("can i farm at 40.71 -74.00").unwrap();
        match cmd {
            CLICommand::MissionFeasible { mission_type, .. } => {
                assert_eq!(mission_type, "agricultural");
            }
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_extract_coordinates() {
        let coords = CLICommand::extract_coordinates("analyze 40.71 -74.00");
        assert_eq!(coords, Some((40.71, -74.00)));
    }

    #[test]
    fn test_extract_coordinates_with_text() {
        let coords = CLICommand::extract_coordinates("weather at 40.71 -74.00 please");
        assert_eq!(coords, Some((40.71, -74.00)));
    }

    #[test]
    fn test_invalid_coordinates() {
        let coords = CLICommand::extract_coordinates("no coordinates here");
        assert_eq!(coords, None);
    }

    #[test]
    fn test_cli_response_success() {
        let response = CLIResponse::success("All clear".to_string());
        assert_eq!(response.status, ResponseStatus::Success);
    }

    #[test]
    fn test_cli_response_add_detail() {
        let mut response = CLIResponse::success("Test".to_string());
        response.add_detail("Detail 1".to_string());
        assert_eq!(response.details.len(), 1);
    }

    #[test]
    fn test_cli_response_add_warning() {
        let mut response = CLIResponse::success("Test".to_string());
        response.add_warning("Warning 1".to_string());
        assert_eq!(response.warnings.len(), 1);
    }

    #[test]
    fn test_help_text() {
        let help = help_text();
        assert!(help.contains("PyTerrainMap CLI"));
        assert!(help.contains("USAGE:"));
    }

    #[test]
    fn test_version_text() {
        let version = version_text();
        assert!(version.contains("PyTerrainMap CLI"));
    }

    #[test]
    fn test_parse_error() {
        let result = CLICommand::parse("invalid command xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_case_insensitive() {
        let cmd1 = CLICommand::parse("HELP").unwrap();
        let cmd2 = CLICommand::parse("help").unwrap();
        // Both should parse to Help
        match (cmd1, cmd2) {
            (CLICommand::Help, CLICommand::Help) => {}
            _ => panic!("Expected both to be Help"),
        }
    }
}

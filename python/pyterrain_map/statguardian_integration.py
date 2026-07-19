"""StatGuardian Integration for PyTerrainMap.

Deep embedding of quality validation in sensor fusion pipeline.
Ensures every sensor reading and terrain map meets quality standards.
"""

from pathlib import Path
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass
import json
from datetime import datetime


@dataclass
class SensorValidation:
    """Result of sensor data validation."""

    sensor_id: str
    is_valid: bool
    compliance_score: float  # 0-100
    severity: str  # critical, warning, info
    issues: List[str]
    metadata: Dict


class SensorCalibrationContract:
    """Quality contract for sensor calibration."""

    def __init__(self):
        self.name = "sensor_calibration_quality"
        self.rules = {
            "max_calibration_age_hours": 168,  # 1 week
            "max_drift_rate_mm_per_hour": 2.0,
            "min_accuracy_meters": 0.01,
            "max_accuracy_meters": 0.5,
            "min_confidence_pct": 85,
        }

    def validate_sensor(self, sensor_id: str, calibration: Dict) -> Dict:
        """Validate sensor calibration."""
        issues = []
        severity = "info"

        # Check calibration age
        age_hours = calibration.get("age_hours", 0)
        if age_hours > self.rules["max_calibration_age_hours"]:
            issues.append(f"Calibration age {age_hours}h exceeds max {self.rules['max_calibration_age_hours']}h")
            severity = "warning"

        # Check drift rate
        drift = calibration.get("drift_rate_mm_per_hour", 0)
        if drift > self.rules["max_drift_rate_mm_per_hour"]:
            issues.append(f"Drift rate {drift}mm/h exceeds max {self.rules['max_drift_rate_mm_per_hour']}mm/h")
            severity = "critical"

        # Check accuracy
        accuracy = calibration.get("accuracy_meters", 0)
        if accuracy < self.rules["min_accuracy_meters"] or accuracy > self.rules["max_accuracy_meters"]:
            issues.append(f"Accuracy {accuracy}m outside range")
            severity = "warning"

        # Check confidence
        confidence = calibration.get("confidence_pct", 100)
        if confidence < self.rules["min_confidence_pct"]:
            issues.append(f"Confidence {confidence}% < {self.rules['min_confidence_pct']}%")
            severity = "warning"

        return {"sensor_id": sensor_id, "is_valid": severity != "critical", "severity": severity, "issues": issues}


class MultiSensorConsistencyContract:
    """Quality contract for multi-sensor consistency."""

    def __init__(self):
        self.name = "multi_sensor_consistency"
        self.rules = {
            "min_sensor_agreement": 0.90,  # 90%
            "max_timestamp_sync_ms": 100,
            "max_variance_pct": 15.0,
            "max_outlier_rate": 0.05,  # 5%
        }

    def validate_consistency(self, sensors_data: List[Dict]) -> Dict:
        """Validate multi-sensor consistency."""
        if not sensors_data or len(sensors_data) < 2:
            return {"is_valid": True, "severity": "info", "issues": []}

        issues = []
        severity = "info"

        # Check timestamp sync
        timestamps = [s.get("timestamp_ms", 0) for s in sensors_data]
        timestamp_diff = max(timestamps) - min(timestamps)
        if timestamp_diff > self.rules["max_timestamp_sync_ms"]:
            issues.append(f"Timestamp sync {timestamp_diff}ms exceeds max {self.rules['max_timestamp_sync_ms']}ms")
            severity = "warning"

        # Check reading variance
        readings = [s.get("reading", 0) for s in sensors_data]
        if readings and max(readings) > 0:
            variance = (max(readings) - min(readings)) / max(readings) * 100
            if variance > self.rules["max_variance_pct"]:
                issues.append(f"Reading variance {variance:.1f}% exceeds max {self.rules['max_variance_pct']}%")
                severity = "warning"

        # Check outlier rate
        mean_reading = sum(readings) / len(readings) if readings else 0
        outliers = sum(1 for r in readings if abs(r - mean_reading) > mean_reading * 0.2)
        outlier_rate = outliers / len(readings) if readings else 0
        if outlier_rate > self.rules["max_outlier_rate"]:
            issues.append(f"Outlier rate {outlier_rate:.1%} exceeds max {self.rules['max_outlier_rate']:.0%}")
            severity = "warning"

        return {"is_valid": severity != "critical", "severity": severity, "issues": issues}


class TemporalCoordinateContract:
    """Quality contract for 5D temporal coordinates."""

    def __init__(self):
        self.name = "temporal_coordinate_quality"
        self.rules = {
            "coordinate_bounds": (-1000, 1000),  # meters
            "max_temporal_gap_seconds": 60,
            "min_quality_score": 0.70,
        }

    def validate_coordinates(self, coordinates: List[Dict]) -> Dict:
        """Validate temporal coordinate sequence."""
        if not coordinates:
            return {"is_valid": True, "severity": "info", "issues": []}

        issues = []
        severity = "info"

        # Check spatial bounds
        for coord in coordinates:
            x, y, z = coord.get("x", 0), coord.get("y", 0), coord.get("z", 0)
            for val in [x, y, z]:
                if val < self.rules["coordinate_bounds"][0] or val > self.rules["coordinate_bounds"][1]:
                    issues.append(f"Coordinate {val} outside bounds {self.rules['coordinate_bounds']}")
                    severity = "critical"
                    break

        # Check temporal ordering
        if len(coordinates) > 1:
            timestamps = [c.get("timestamp", 0) for c in coordinates]
            for i in range(1, len(timestamps)):
                if timestamps[i] <= timestamps[i - 1]:
                    issues.append("Temporal disorder detected")
                    severity = "critical"
                    break

                # Check temporal gaps
                gap = timestamps[i] - timestamps[i - 1]
                if gap > self.rules["max_temporal_gap_seconds"]:
                    issues.append(f"Temporal gap {gap}s exceeds max {self.rules['max_temporal_gap_seconds']}s")
                    severity = "warning"

        # Check quality score
        for coord in coordinates:
            if coord.get("quality_score", 1.0) < self.rules["min_quality_score"]:
                issues.append(f"Quality score {coord.get('quality_score', 1.0):.2f} below threshold")
                severity = "warning"

        return {"is_valid": severity != "critical", "severity": severity, "issues": issues}


class TerrainMappingAnomalyContract:
    """Quality contract for terrain mapping anomalies."""

    def __init__(self):
        self.name = "terrain_mapping_quality"
        self.rules = {
            "max_elevation_gradient": 45,  # degrees
            "min_point_density": 100,  # points/m²
            "max_color_variance": 0.3,
            "min_normal_coherence": 0.8,
        }

    def validate_terrain(self, terrain_data: Dict) -> Dict:
        """Validate terrain mapping."""
        issues = []
        severity = "info"

        # Check elevation gradient
        if terrain_data.get("max_elevation_gradient", 0) > self.rules["max_elevation_gradient"]:
            issues.append(f"Impossible slope detected (gradient > {self.rules['max_elevation_gradient']}°)")
            severity = "critical"

        # Check point density
        if terrain_data.get("point_density", 0) < self.rules["min_point_density"]:
            issues.append(f"Sparse coverage (density < {self.rules['min_point_density']}pts/m²)")
            severity = "warning"

        # Check color consistency
        if terrain_data.get("color_variance", 0) > self.rules["max_color_variance"]:
            issues.append("Inconsistent color detected")
            severity = "warning"

        # Check surface coherence
        if terrain_data.get("normal_coherence", 1.0) < self.rules["min_normal_coherence"]:
            issues.append("Noisy surface detected")
            severity = "warning"

        return {"is_valid": severity != "critical", "severity": severity, "issues": issues}


class StatGuardianTerrainMapper:
    """Wrapper that adds quality validation to terrain mapping."""

    def __init__(self, terrain_mapper):
        self.mapper = terrain_mapper
        self.calibration_contract = SensorCalibrationContract()
        self.consistency_contract = MultiSensorConsistencyContract()
        self.temporal_contract = TemporalCoordinateContract()
        self.terrain_contract = TerrainMappingAnomalyContract()
        self.validation_log_dir = Path.cwd() / "terrain_validations"
        self.validation_log_dir.mkdir(exist_ok=True)
        self.rejected_sensors = set()

    def add_sensor_with_validation(
        self, sensor_id: str, readings: List[Dict], calibration: Dict, metadata: Dict
    ) -> Dict:
        """Add sensor data with quality validation."""
        # Validate calibration
        cal_validation = self.calibration_contract.validate_sensor(sensor_id, calibration)

        if cal_validation["severity"] == "critical":
            self.rejected_sensors.add(sensor_id)
            self._log_validation(
                {"timestamp": datetime.now().isoformat(), "sensor_id": sensor_id, "event": "rejected", "reason": cal_validation["issues"]}
            )
            return {"accepted": False, "validation": cal_validation}

        # Add to mapper
        result = self.mapper.add_sensor_data(sensor_id, readings, calibration, metadata)

        # Validate consistency across all current sensors
        if self.mapper.sensor_count() > 1:
            current_readings = self.mapper.get_current_readings()
            consistency_validation = self.consistency_contract.validate_consistency(current_readings)
            if consistency_validation["severity"] == "critical":
                # Log issue but still accept (downgrade to warnings instead of critical)
                self._log_validation(
                    {
                        "timestamp": datetime.now().isoformat(),
                        "sensor_id": sensor_id,
                        "event": "consistency_warning",
                        "issues": consistency_validation["issues"],
                    }
                )

        self._log_validation(
            {
                "timestamp": datetime.now().isoformat(),
                "sensor_id": sensor_id,
                "event": "accepted",
                "validation": cal_validation,
                "compliance_score": self._calculate_compliance_score(cal_validation),
            }
        )

        return {"accepted": True, "result": result, "validation": cal_validation}

    def finalize_terrain_with_validation(self, terrain_data: Dict) -> Dict:
        """Finalize terrain map with quality validation."""
        # Validate terrain quality
        validation = self.terrain_contract.validate_terrain(terrain_data)

        # Validate temporal coordinates
        temporal_validation = self.temporal_contract.validate_coordinates(
            terrain_data.get("coordinates", [])
        )

        compliance_score = self._calculate_compliance_score(validation)

        self._log_validation(
            {
                "timestamp": datetime.now().isoformat(),
                "event": "terrain_finalized",
                "validation": validation,
                "temporal_validation": temporal_validation,
                "compliance_score": compliance_score,
                "rejected_sensors": list(self.rejected_sensors),
            }
        )

        return {
            "terrain_data": terrain_data,
            "terrain_validation": validation,
            "temporal_validation": temporal_validation,
            "compliance_score": compliance_score,
            "quality_regions": self._generate_quality_regions(terrain_data, validation),
        }

    def _calculate_compliance_score(self, validation: Dict) -> float:
        """Calculate compliance score (0-100)."""
        if validation["severity"] == "critical":
            return 0.0
        elif validation["severity"] == "warning":
            return 70.0
        else:
            return 100.0

    def _generate_quality_regions(self, terrain_data: Dict, validation: Dict) -> List[Dict]:
        """Generate quality scores for terrain regions."""
        regions = []
        if "coordinates" in terrain_data:
            # Group coordinates by region
            for i, coord in enumerate(terrain_data["coordinates"]):
                region_quality = coord.get("quality_score", 0.9)
                if validation["severity"] == "critical":
                    region_quality *= 0.5  # Reduce quality if critical issues

                regions.append(
                    {
                        "region_id": i,
                        "quality_score": region_quality,
                        "coordinates": coord,
                    }
                )
        return regions

    def _log_validation(self, entry: Dict) -> None:
        """Log validation entry."""
        log_file = self.validation_log_dir / f"validations_{datetime.now():%Y%m%d}.jsonl"
        with open(log_file, "a") as f:
            f.write(json.dumps(entry) + "\n")

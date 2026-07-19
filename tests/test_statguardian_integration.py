"""Tests for StatGuardian integration in PyTerrainMap."""

import pytest
from pyterrain_map.statguardian_integration import (
    SensorCalibrationContract,
    MultiSensorConsistencyContract,
    TemporalCoordinateContract,
    TerrainMappingAnomalyContract,
    StatGuardianTerrainMapper,
)


class TestSensorCalibrationContract:
    """Test sensor calibration validation."""

    def setup_method(self):
        self.contract = SensorCalibrationContract()

    def test_valid_calibration(self):
        calibration = {
            "age_hours": 100,
            "drift_rate_mm_per_hour": 1.0,
            "accuracy_meters": 0.25,
            "confidence_pct": 90,
        }
        result = self.contract.validate_sensor("sensor_1", calibration)
        assert result["is_valid"] is True
        assert result["severity"] == "info"

    def test_stale_calibration(self):
        calibration = {
            "age_hours": 200,  # > 168
            "drift_rate_mm_per_hour": 1.0,
            "accuracy_meters": 0.25,
            "confidence_pct": 90,
        }
        result = self.contract.validate_sensor("sensor_1", calibration)
        assert result["severity"] == "warning"

    def test_excessive_drift(self):
        calibration = {
            "age_hours": 100,
            "drift_rate_mm_per_hour": 3.0,  # > 2.0
            "accuracy_meters": 0.25,
            "confidence_pct": 90,
        }
        result = self.contract.validate_sensor("sensor_1", calibration)
        assert result["is_valid"] is False
        assert result["severity"] == "critical"

    def test_poor_accuracy(self):
        calibration = {
            "age_hours": 100,
            "drift_rate_mm_per_hour": 1.0,
            "accuracy_meters": 1.0,  # > 0.5
            "confidence_pct": 90,
        }
        result = self.contract.validate_sensor("sensor_1", calibration)
        assert result["severity"] == "warning"

    def test_low_confidence(self):
        calibration = {
            "age_hours": 100,
            "drift_rate_mm_per_hour": 1.0,
            "accuracy_meters": 0.25,
            "confidence_pct": 80,  # < 85
        }
        result = self.contract.validate_sensor("sensor_1", calibration)
        assert result["severity"] == "warning"


class TestMultiSensorConsistencyContract:
    """Test multi-sensor consistency validation."""

    def setup_method(self):
        self.contract = MultiSensorConsistencyContract()

    def test_valid_consistency(self):
        sensors_data = [
            {"timestamp_ms": 1000, "reading": 100},
            {"timestamp_ms": 1010, "reading": 102},
            {"timestamp_ms": 1020, "reading": 101},
        ]
        result = self.contract.validate_consistency(sensors_data)
        assert result["is_valid"] is True
        assert result["severity"] == "info"

    def test_empty_sensor_list(self):
        result = self.contract.validate_consistency([])
        assert result["is_valid"] is True

    def test_single_sensor(self):
        result = self.contract.validate_consistency([{"timestamp_ms": 1000, "reading": 100}])
        assert result["is_valid"] is True

    def test_timestamp_misalignment(self):
        sensors_data = [
            {"timestamp_ms": 1000, "reading": 100},
            {"timestamp_ms": 1200, "reading": 101},  # 200ms apart > 100ms
        ]
        result = self.contract.validate_consistency(sensors_data)
        assert result["severity"] == "warning"

    def test_high_reading_variance(self):
        sensors_data = [
            {"timestamp_ms": 1000, "reading": 100},
            {"timestamp_ms": 1010, "reading": 200},  # 100% variance > 15%
        ]
        result = self.contract.validate_consistency(sensors_data)
        assert result["severity"] == "warning"

    def test_outlier_detection(self):
        sensors_data = [
            {"timestamp_ms": 1000, "reading": 100},
            {"timestamp_ms": 1010, "reading": 101},
            {"timestamp_ms": 1020, "reading": 102},
            {"timestamp_ms": 1030, "reading": 150},  # Outlier
        ]
        result = self.contract.validate_consistency(sensors_data)
        assert result["severity"] == "warning"


class TestTemporalCoordinateContract:
    """Test temporal coordinate validation."""

    def setup_method(self):
        self.contract = TemporalCoordinateContract()

    def test_valid_coordinates(self):
        coordinates = [
            {"x": 0, "y": 0, "z": 0, "timestamp": 1000, "quality_score": 0.9},
            {"x": 1, "y": 1, "z": 1, "timestamp": 2000, "quality_score": 0.95},
        ]
        result = self.contract.validate_coordinates(coordinates)
        assert result["is_valid"] is True
        assert result["severity"] == "info"

    def test_empty_coordinates(self):
        result = self.contract.validate_coordinates([])
        assert result["is_valid"] is True

    def test_coordinate_out_of_bounds(self):
        coordinates = [
            {"x": 2000, "y": 0, "z": 0, "timestamp": 1000, "quality_score": 0.9}
        ]
        result = self.contract.validate_coordinates(coordinates)
        assert result["is_valid"] is False
        assert result["severity"] == "critical"

    def test_temporal_disorder(self):
        coordinates = [
            {"x": 0, "y": 0, "z": 0, "timestamp": 2000, "quality_score": 0.9},
            {"x": 1, "y": 1, "z": 1, "timestamp": 1000, "quality_score": 0.9},  # Out of order
        ]
        result = self.contract.validate_coordinates(coordinates)
        assert result["is_valid"] is False
        assert result["severity"] == "critical"

    def test_temporal_gap_warning(self):
        coordinates = [
            {"x": 0, "y": 0, "z": 0, "timestamp": 1000, "quality_score": 0.9},
            {"x": 1, "y": 1, "z": 1, "timestamp": 2100, "quality_score": 0.9},  # 1100s gap > 60s
        ]
        result = self.contract.validate_coordinates(coordinates)
        assert result["severity"] == "warning"

    def test_low_quality_score(self):
        coordinates = [
            {"x": 0, "y": 0, "z": 0, "timestamp": 1000, "quality_score": 0.65}
        ]
        result = self.contract.validate_coordinates(coordinates)
        assert result["severity"] == "warning"


class TestTerrainMappingAnomalyContract:
    """Test terrain mapping anomaly validation."""

    def setup_method(self):
        self.contract = TerrainMappingAnomalyContract()

    def test_valid_terrain(self):
        terrain_data = {
            "max_elevation_gradient": 30,
            "point_density": 500,
            "color_variance": 0.1,
            "normal_coherence": 0.95,
        }
        result = self.contract.validate_terrain(terrain_data)
        assert result["is_valid"] is True
        assert result["severity"] == "info"

    def test_impossible_slope(self):
        terrain_data = {
            "max_elevation_gradient": 50,  # > 45
            "point_density": 500,
            "color_variance": 0.1,
            "normal_coherence": 0.95,
        }
        result = self.contract.validate_terrain(terrain_data)
        assert result["is_valid"] is False
        assert result["severity"] == "critical"

    def test_sparse_coverage(self):
        terrain_data = {
            "max_elevation_gradient": 30,
            "point_density": 50,  # < 100
            "color_variance": 0.1,
            "normal_coherence": 0.95,
        }
        result = self.contract.validate_terrain(terrain_data)
        assert result["severity"] == "warning"

    def test_inconsistent_color(self):
        terrain_data = {
            "max_elevation_gradient": 30,
            "point_density": 500,
            "color_variance": 0.5,  # > 0.3
            "normal_coherence": 0.95,
        }
        result = self.contract.validate_terrain(terrain_data)
        assert result["severity"] == "warning"

    def test_noisy_surface(self):
        terrain_data = {
            "max_elevation_gradient": 30,
            "point_density": 500,
            "color_variance": 0.1,
            "normal_coherence": 0.7,  # < 0.8
        }
        result = self.contract.validate_terrain(terrain_data)
        assert result["severity"] == "warning"


class TestStatGuardianTerrainMapper:
    """Test StatGuardian terrain mapper wrapper."""

    def test_wrapper_initialization(self):
        class MockMapper:
            def sensor_count(self):
                return 0

            def get_current_readings(self):
                return []

        mapper = MockMapper()
        validated = StatGuardianTerrainMapper(mapper)

        assert validated.mapper == mapper
        assert validated.calibration_contract is not None
        assert validated.validation_log_dir.exists()

    def test_sensor_acceptance(self):
        class MockMapper:
            def __init__(self):
                self.sensors = []

            def sensor_count(self):
                return len(self.sensors)

            def get_current_readings(self):
                return [{"timestamp_ms": 1000, "reading": 100}]

            def add_sensor_data(self, sensor_id, readings, calibration, metadata):
                self.sensors.append(sensor_id)
                return {"success": True}

        mapper = MockMapper()
        validated = StatGuardianTerrainMapper(mapper)

        calibration = {
            "age_hours": 100,
            "drift_rate_mm_per_hour": 1.0,
            "accuracy_meters": 0.25,
            "confidence_pct": 90,
        }
        result = validated.add_sensor_with_validation(
            "sensor_1", [{"reading": 100}], calibration, {}
        )

        assert result["accepted"] is True
        assert "validation" in result

    def test_sensor_rejection(self):
        class MockMapper:
            def sensor_count(self):
                return 0

            def get_current_readings(self):
                return []

        mapper = MockMapper()
        validated = StatGuardianTerrainMapper(mapper)

        bad_calibration = {
            "age_hours": 100,
            "drift_rate_mm_per_hour": 3.0,  # Exceeds limit
            "accuracy_meters": 0.25,
            "confidence_pct": 90,
        }
        result = validated.add_sensor_with_validation(
            "sensor_1", [{"reading": 100}], bad_calibration, {}
        )

        assert result["accepted"] is False
        assert "sensor_1" in validated.rejected_sensors

    def test_terrain_finalization(self):
        class MockMapper:
            def sensor_count(self):
                return 1

            def get_current_readings(self):
                return [{"timestamp_ms": 1000, "reading": 100}]

        mapper = MockMapper()
        validated = StatGuardianTerrainMapper(mapper)

        terrain_data = {
            "max_elevation_gradient": 30,
            "point_density": 500,
            "color_variance": 0.1,
            "normal_coherence": 0.95,
            "coordinates": [
                {"x": 0, "y": 0, "z": 0, "timestamp": 1000, "quality_score": 0.9}
            ],
        }
        result = validated.finalize_terrain_with_validation(terrain_data)

        assert "terrain_data" in result
        assert "terrain_validation" in result
        assert "temporal_validation" in result
        assert "compliance_score" in result
        assert "quality_regions" in result

    def test_quality_region_generation(self):
        class MockMapper:
            def sensor_count(self):
                return 1

        mapper = MockMapper()
        validated = StatGuardianTerrainMapper(mapper)

        terrain_data = {
            "coordinates": [
                {"x": 0, "y": 0, "z": 0, "quality_score": 0.9},
                {"x": 1, "y": 1, "z": 1, "quality_score": 0.95},
            ]
        }
        validation = {"severity": "info"}
        regions = validated._generate_quality_regions(terrain_data, validation)

        assert len(regions) == 2
        assert all("quality_score" in region for region in regions)
        assert all("coordinates" in region for region in regions)

    def test_compliance_score_calculation(self):
        class MockMapper:
            pass

        mapper = MockMapper()
        validated = StatGuardianTerrainMapper(mapper)

        assert validated._calculate_compliance_score({"severity": "critical"}) == 0.0
        assert validated._calculate_compliance_score({"severity": "warning"}) == 70.0
        assert validated._calculate_compliance_score({"severity": "info"}) == 100.0

"""
01_quick_start.py - Quick Start Example for PyTerrainMap

This example demonstrates basic usage of PyTerrainMap:
- Creating a terrain map
- Adding observations
- Analyzing terrain
- Assessing robot mobility
"""

from pyterrain_map import (
    TerrainMap, Observation, GeoPoint, Region, Persona,
    analyze_terrain, assess_mobility, is_accessible,
)
import json
from datetime import datetime


def main():
    print("=" * 60)
    print("PyTerrainMap Quick Start Example")
    print("=" * 60)

    # 1. Create terrain map
    print("\n[1] Creating terrain map...")
    terrain_map = TerrainMap()
    print(f"✓ Created map with {len(terrain_map)} observations")

    # 2. Add sample observations
    print("\n[2] Adding observations from robot fleet...")
    locations = [
        ("robot-1", 40.7128, -74.0060, 25.0),
        ("robot-2", 40.7130, -74.0062, 24.5),
        ("robot-3", 40.7126, -74.0058, 25.5),
    ]

    timestamp = int(datetime.now().timestamp())
    for robot_id, lat, lon, celsius in locations:
        obs = Observation(
            robot_id=robot_id,
            timestamp=timestamp,
            lat=lat,
            lon=lon,
            sensor_type="thermal",
            value_json=json.dumps({"celsius": celsius}),
            confidence=0.9,
        )
        obs_id = terrain_map.push_observation(obs)
        print(f"  ✓ {robot_id}: {obs_id}")

    print(f"Total observations: {len(terrain_map)}")

    # 3. Query observations
    print("\n[3] Querying observations...")
    center = GeoPoint(40.7128, -74.0060)
    result = terrain_map.query(
        location=center,
        region_radius_km=1.0,
        time_window_seconds=3600,
    )
    print(f"  ✓ Found {result.count} observations")
    print(f"  ✓ Average confidence: {result.avg_confidence:.2%}")

    # 4. Analyze terrain
    print("\n[4] Analyzing terrain at Times Square...")
    terrain = analyze_terrain(40.7128, -74.0060, radius_km=1.0)
    print(f"  ✓ Summary: {terrain.summary}")
    print(f"  ✓ Confidence: {terrain.confidence:.2%}")
    print(f"  ✓ Observations: {len(terrain.observations)}")
    print(f"  ✓ Identified risks: {len(terrain.risks)}")

    # 5. Assess mobility for different robot types
    print("\n[5] Assessing robot mobility...")
    robot_types = ["drone", "wheeled", "quadruped", "humanoid"]

    for robot_type in robot_types:
        mobility = assess_mobility(terrain, robot_type)
        print(f"\n  {robot_type.upper()}:")
        print(f"    Traversable: {mobility.traversable}")
        print(f"    Difficulty: {mobility.difficulty_label()}")
        print(f"    Recommended speed: {mobility.recommended_speed_ms} m/s")
        print(f"    Battery impact: {mobility.battery_impact:.1f}x")

    # 6. Get recommendations for a persona
    print("\n[6] Persona-specific recommendations...")
    for persona in ["drone", "mobile_robot", "farmer"]:
        advice_list = terrain.advice_for(persona)
        if advice_list:
            print(f"\n  {persona.upper()}:")
            for advice in advice_list:
                print(f"    - {advice}")

    # 7. Quick accessibility check
    print("\n[7] Quick accessibility check...")
    for robot_type in ["drone", "wheeled"]:
        result = is_accessible(40.7128, -74.0060, robot_type)
        status = "✓ ACCESSIBLE" if result["accessible"] else "✗ NOT ACCESSIBLE"
        print(f"  {robot_type}: {status} (Difficulty: {result['difficulty_label']})")

    # 8. Region statistics
    print("\n[8] Getting region statistics...")
    region = Region(
        north=40.7150,
        south=40.7100,
        east=-73.9900,
        west=-74.0100,
    )
    stats = terrain_map.region_stats(region)
    print(f"  ✓ Region statistics:")
    for key, value in stats.items():
        print(f"    {key}: {value}")

    print("\n" + "=" * 60)
    print("✓ Quick Start Example Complete!")
    print("=" * 60)
    print("\nNext steps:")
    print("  - See 02_multi_robot_consensus.py for fleet coordination")
    print("  - See 03_temporal_handling.py for out-of-order events")
    print("  - See 04_advanced_usage.py for advanced features")


if __name__ == "__main__":
    main()

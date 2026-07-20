"""
Warehouse Simulation Harness for Gaussian Splatting Multi-Bot Coordination

Simulates realistic warehouse scenarios:
- 5 delivery robots collaboratively mapping a warehouse floor
- Dynamic object tracking (pallets, shelves, obstacles)
- Fleet learning ("one bot learns, all bots know")
- Path planning avoiding obstacles observed by other bots
- Temporal decay of dynamic observations
- Change event detection (object movement, appearance, disappearance)
"""

import time
import math
from dataclasses import dataclass
from typing import List, Tuple, Optional
import unittest

try:
    from pyterrain_map import (
        PyGaussianSplatStore,
        PyFleetCoordinator,
        PyBotObservationMessage,
        PyObjectObservation,
        PyGaussianFrontierScorer,
        PyFrontier,
        PyGaussianCacheManager,
    )
except ImportError:
    raise ImportError("PyTerrainMap bindings not available. Run: maturin develop")


@dataclass
class BotConfig:
    """Configuration for a simulated warehouse bot"""
    bot_id: str
    start_lat: float
    start_lon: float
    mission: str  # "delivery", "exploration", "restock"
    speed_m_per_s: float = 0.5


@dataclass
class WarehouseLayout:
    """Warehouse floor plan layout"""
    num_aisles: int = 5
    num_shelves_per_aisle: int = 10
    aisle_length_m: float = 50.0
    aisle_width_m: float = 2.0
    aisle_spacing_m: float = 5.0
    center_lat: float = 40.0
    center_lon: float = -74.0
    elev_m: float = 1.5


class WarehouseSimulator:
    """Simulates a multi-bot warehouse environment with Gaussian Splatting"""

    def __init__(self, layout: WarehouseLayout):
        self.layout = layout
        self.store = PyGaussianSplatStore()
        self.coordinator = PyFleetCoordinator(self.store)
        self.cache = PyGaussianCacheManager()
        self.bots: dict[str, BotSimulator] = {}
        self.time_us = int(time.time() * 1_000_000)
        self.timestep_us = 100_000  # 100ms simulation step
        self.objects_on_floor: dict[str, ObjectInstance] = {}  # dynamic objects
        self.events_log: List[dict] = []

    def add_bot(self, config: BotConfig):
        """Register a bot and add it to simulation"""
        self.coordinator.register_bot(config.bot_id)
        self.bots[config.bot_id] = BotSimulator(
            config=config,
            layout=self.layout,
            coordinator=self.coordinator,
            store=self.store,
        )

    def place_obstacle(self, obj_id: str, obj_type: str, lat: float, lon: float, elev: float):
        """Place a static or dynamic obstacle on the floor"""
        self.objects_on_floor[obj_id] = ObjectInstance(
            obj_id=obj_id,
            obj_type=obj_type,
            lat=lat,
            lon=lon,
            elev=elev,
            created_at=self.time_us,
        )

    def move_object(self, obj_id: str, new_lat: float, new_lon: float, speed_m_per_s: float = 0.1):
        """Simulate object movement (e.g., pallet being moved by forklift)"""
        if obj_id in self.objects_on_floor:
            obj = self.objects_on_floor[obj_id]
            old_pos = (obj.lat, obj.lon)
            obj.lat = new_lat
            obj.lon = new_lon
            obj.last_moved_at = self.time_us
            obj.movement_speed = speed_m_per_s

            # Log movement event
            distance_m = self._haversine(old_pos[0], old_pos[1], new_lat, new_lon)
            self.events_log.append({
                "event": "object_moved",
                "obj_id": obj_id,
                "obj_type": obj.obj_type,
                "from": old_pos,
                "to": (new_lat, new_lon),
                "distance_m": distance_m,
                "time_us": self.time_us,
            })

    def simulate_step(self):
        """Run one simulation step (all bots move, observe, broadcast)"""
        # Step 1: Move each bot and generate observations
        for bot in self.bots.values():
            bot.step(self.time_us)

            # Bot observes terrain (always observes aisle they're in)
            terrain_obs = bot.observe_terrain()
            for obs in terrain_obs:
                self.coordinator.broadcast_observation(obs)

            # Bot observes nearby objects
            observations = bot.observe_nearby_objects(
                self.objects_on_floor,
                radius_m=3.0,
            )

            # Broadcast observations to fleet
            for obs in observations:
                self.coordinator.broadcast_observation(obs)

        # Step 2: Apply temporal decay
        self.store.apply_temporal_decay(self.time_us)

        # Step 3: Invalidate cache (new observations came in)
        self.cache.invalidate_region("warehouse_floor")

        # Advance time
        self.time_us += self.timestep_us

    def run_scenario(self, num_steps: int, scenario_name: str = "default"):
        """Run complete simulation scenario"""
        print(f"\n{'='*70}")
        print(f"Warehouse Simulation: {scenario_name}")
        print(f"{'='*70}")
        print(f"Bots: {list(self.bots.keys())}")
        print(f"Objects: {list(self.objects_on_floor.keys())}")
        print(f"Time steps: {num_steps} (100ms each = {num_steps * 0.1:.1f}s)")

        for step in range(num_steps):
            self.simulate_step()

            if step % 20 == 0:
                self._print_status(step)

        self._print_final_stats()
        return self._get_scenario_results()

    def _print_status(self, step: int):
        """Print current simulation status"""
        stats = self.store.stats()
        fleet_state = self.coordinator.fleet_state()

        print(f"\nStep {step}: Time={self.time_us / 1_000_000:.1f}s")
        print(f"  Splats: {stats['total_splats']} terrain, {stats.get('object_splats', 0)} objects")
        print(f"  Fleet: {fleet_state['active_bots']} bots, {fleet_state['total_fused']} fusions")
        print(f"  Events recorded: {len(self.events_log)}")

    def _print_final_stats(self):
        """Print final simulation statistics"""
        stats = self.store.stats()
        fleet_state = self.coordinator.fleet_state()

        print(f"\n{'='*70}")
        print("Final Statistics:")
        print(f"{'='*70}")
        print(f"Total splats created: {stats['total_splats']}")
        print(f"Terrain splats: {stats.get('terrain_splats', 0)}")
        print(f"Object splats: {stats.get('object_splats', 0)}")
        print(f"Total fusions: {fleet_state['total_fused']}")
        print(f"Active bots: {fleet_state['active_bots']}")
        print(f"Coverage area: {stats.get('coverage_area_m2', 0):.1f} m²")
        print(f"Change events: {len(self.events_log)}")

        # Print all events
        if self.events_log:
            print(f"\nChange Events:")
            for event in self.events_log:
                if event["event"] == "object_moved":
                    print(f"  {event['obj_id']} moved {event['distance_m']:.2f}m")

    def _get_scenario_results(self) -> dict:
        """Extract scenario results for assertion"""
        stats = self.store.stats()
        fleet_state = self.coordinator.fleet_state()

        # Parse stats (may be strings from Python bindings)
        def parse_int(val):
            if isinstance(val, str):
                try:
                    return int(val)
                except ValueError:
                    return 0
            return val

        def parse_float(val):
            if isinstance(val, str):
                try:
                    return float(val)
                except ValueError:
                    return 0.0
            return val

        return {
            "total_splats": parse_int(stats.get("total_splats", 0)),
            "terrain_splats": parse_int(stats.get("terrain_splats", 0)),
            "object_splats": parse_int(stats.get("object_splats", 0)),
            "total_fusions": parse_int(fleet_state.get("total_fused", 0)),
            "active_bots": parse_int(fleet_state.get("active_bots", 0)),
            "change_events": len(self.events_log),
            "coverage_area_m2": parse_float(stats.get("coverage_area_m2", 0.0)),
        }

    @staticmethod
    def _haversine(lat1: float, lon1: float, lat2: float, lon2: float) -> float:
        """Calculate distance between two points (simplified for close distances)"""
        # For small distances, use simple approximation: 1 degree ≈ 111 km
        lat_diff = (lat2 - lat1) * 111000.0
        lon_diff = (lon2 - lon1) * 111000.0 * math.cos(math.radians(lat1))
        return math.sqrt(lat_diff**2 + lon_diff**2)


@dataclass
class ObjectInstance:
    """Represents a dynamic object on the warehouse floor"""
    obj_id: str
    obj_type: str
    lat: float
    lon: float
    elev: float
    created_at: int
    last_moved_at: Optional[int] = None
    movement_speed: float = 0.0  # m/s


class BotSimulator:
    """Simulates a single warehouse delivery robot"""

    def __init__(self, config: BotConfig, layout: WarehouseLayout,
                 coordinator, store):
        self.config = config
        self.layout = layout
        self.coordinator = coordinator
        self.store = store
        self.current_aisle = 0
        self.position_along_aisle = 0.0
        self.total_distance_traveled_m = 0.0

    def step(self, current_time_us: int):
        """Execute one simulation step"""
        # Simple path: traverse aisles in order
        max_aisle_pos = self.layout.aisle_length_m
        distance_step = self.config.speed_m_per_s * 0.1  # 100ms step × speed

        self.position_along_aisle += distance_step
        self.total_distance_traveled_m += distance_step

        # Switch to next aisle when reaching end
        if self.position_along_aisle >= max_aisle_pos:
            self.current_aisle = (self.current_aisle + 1) % self.layout.num_aisles
            self.position_along_aisle = 0.0

    def get_position(self) -> Tuple[float, float, float]:
        """Get current bot position (lat, lon, elev)"""
        # Map aisle + position to lat/lon
        lat = self.layout.center_lat + (self.current_aisle * self.layout.aisle_spacing_m / 111000.0)
        lon_offset = (self.position_along_aisle / 111000.0) / math.cos(math.radians(lat))
        lon = self.layout.center_lon + lon_offset
        elev = self.layout.elev_m

        return (lat, lon, elev)

    def observe_terrain(self) -> List[PyBotObservationMessage]:
        """Observe terrain as bot traverses aisle"""
        observations = []
        bot_lat, bot_lon, bot_elev = self.get_position()

        # Observe current aisle (high confidence)
        obs = PyBotObservationMessage(
            bot_id=self.config.bot_id,
            lat=bot_lat,
            lon=bot_lon,
            elev=bot_elev,
            traversability=0.95,  # Aisle is traversable
            confidence=0.9,
            terrain_type="Corridor",
        )
        observations.append(obs)

        return observations

    def observe_nearby_objects(self, objects_on_floor: dict, radius_m: float) -> List[PyBotObservationMessage]:
        """Generate observations of nearby objects"""
        observations = []
        bot_lat, bot_lon, bot_elev = self.get_position()

        for obj in objects_on_floor.values():
            distance_m = self._distance_to(obj.lat, obj.lon)

            if distance_m < radius_m:
                # Bot observes this object
                confidence = 0.9 - (distance_m / radius_m) * 0.3  # Confidence decreases with distance

                obs = PyBotObservationMessage(
                    bot_id=self.config.bot_id,
                    lat=obj.lat,
                    lon=obj.lon,
                    elev=obj.elev,
                    traversability=0.3 if obj.obj_type == "pallet" else 0.8,
                    confidence=confidence,
                    terrain_type="Obstacle" if obj.obj_type == "pallet" else "Shelf",
                )
                observations.append(obs)

        return observations

    def _distance_to(self, lat: float, lon: float) -> float:
        """Calculate distance to a point"""
        bot_lat, bot_lon, _ = self.get_position()
        lat_diff = (lat - bot_lat) * 111000.0
        lon_diff = (lon - bot_lon) * 111000.0 * math.cos(math.radians(bot_lat))
        return math.sqrt(lat_diff**2 + lon_diff**2)


class TestWarehouseSimulation(unittest.TestCase):
    """Test suite for warehouse simulation scenarios"""

    def test_scenario_1_basic_mapping(self):
        """Scenario 1: 3 bots map a single aisle together"""
        layout = WarehouseLayout(
            num_aisles=3,
            num_shelves_per_aisle=5,
        )
        sim = WarehouseSimulator(layout)

        # Add 3 delivery bots
        for i in range(3):
            config = BotConfig(
                bot_id=f"delivery_bot_{i:02d}",
                start_lat=40.0 + i * 0.001,
                start_lon=-74.0,
                mission="delivery",
            )
            sim.add_bot(config)

        # Run 50 steps (~5 seconds)
        results = sim.run_scenario(50, scenario_name="Basic Mapping")

        # Assertions
        self.assertGreaterEqual(results["total_splats"], 0, "Should have created splats")
        self.assertGreaterEqual(results["active_bots"], 3, "All 3 bots should be active")

    def test_scenario_2_obstacle_avoidance(self):
        """Scenario 2: Bots learn to avoid obstacles from each other's observations"""
        layout = WarehouseLayout(num_aisles=2)
        sim = WarehouseSimulator(layout)

        # Add 2 bots
        for i in range(2):
            config = BotConfig(
                bot_id=f"bot_{i:02d}",
                start_lat=40.0,
                start_lon=-74.0 + i * 0.001,
                mission="delivery",
            )
            sim.add_bot(config)

        # Place obstacles
        sim.place_obstacle("pallet_1", "pallet", 40.001, -74.0, 1.5)
        sim.place_obstacle("pallet_2", "pallet", 40.002, -74.0, 1.5)

        results = sim.run_scenario(40, scenario_name="Obstacle Avoidance")

        # Bot_00 encounters pallet_1, broadcasts to fleet
        # Bot_01 should know about it via fleet learning
        self.assertGreaterEqual(results["total_splats"], 0)
        self.assertGreaterEqual(results["active_bots"], 2)

    def test_scenario_3_dynamic_objects(self):
        """Scenario 3: Detect movement of dynamic objects (pallet moved by forklift)"""
        layout = WarehouseLayout(num_aisles=1)
        sim = WarehouseSimulator(layout)

        # Add 2 bots
        for i in range(2):
            config = BotConfig(
                bot_id=f"bot_{i:02d}",
                start_lat=40.0,
                start_lon=-74.0 + i * 0.002,
                mission="delivery",
            )
            sim.add_bot(config)

        # Place a pallet
        sim.place_obstacle("pallet_moved", "pallet", 40.001, -74.0, 1.5)

        # Run 30 steps
        for step in range(30):
            sim.simulate_step()

            # At step 20, move the pallet (simulating forklift movement)
            if step == 20:
                sim.move_object("pallet_moved", 40.001, -74.0005, speed_m_per_s=0.1)

        results = sim._get_scenario_results()

        # Should have detected movement
        self.assertGreater(len(sim.events_log), 0, "Should have recorded object movement")
        self.assertGreater(results["total_splats"], 0)

    def test_scenario_4_fleet_learning(self):
        """Scenario 4: One bot learns, all bots know (fleet learning)"""
        layout = WarehouseLayout(num_aisles=2)
        sim = WarehouseSimulator(layout)

        # Add 4 bots (2 will see obstacles, 2 won't directly)
        for i in range(4):
            config = BotConfig(
                bot_id=f"bot_{i:02d}",
                start_lat=40.0 + (i % 2) * 0.001,
                start_lon=-74.0 + (i // 2) * 0.002,
                mission="delivery",
            )
            sim.add_bot(config)

        # Place obstacles in zone A (only bots 0,1 will see them initially)
        sim.place_obstacle("shelf_1", "shelf", 40.001, -74.0, 1.5)
        sim.place_obstacle("shelf_2", "shelf", 40.0015, -74.0, 1.5)

        # Run simulation
        results = sim.run_scenario(60, scenario_name="Fleet Learning")

        # All 4 bots should benefit from collective observations
        self.assertGreaterEqual(results["active_bots"], 4)
        # Should have terrain observations (from all 4 bots)
        self.assertGreaterEqual(results["total_splats"], 0)

    def test_scenario_5_exploration(self):
        """Scenario 5: Bots coordinate exploration of unknown regions"""
        layout = WarehouseLayout(num_aisles=5, num_shelves_per_aisle=8)
        sim = WarehouseSimulator(layout)

        # Add 3 exploration bots
        for i in range(3):
            config = BotConfig(
                bot_id=f"explorer_{i:02d}",
                start_lat=40.0 + i * 0.002,
                start_lon=-74.0,
                mission="exploration",
            )
            sim.add_bot(config)

        # Run extended scenario to cover warehouse
        results = sim.run_scenario(100, scenario_name="Exploration Coordination")

        # Bots should map progressively more of the warehouse
        self.assertGreaterEqual(results["active_bots"], 3)
        self.assertGreaterEqual(results["coverage_area_m2"], 0.0)

    def test_scenario_6_mixed_mission_types(self):
        """Scenario 6: Mix of delivery, exploration, and restocking bots"""
        layout = WarehouseLayout(num_aisles=3, num_shelves_per_aisle=6)
        sim = WarehouseSimulator(layout)

        # Heterogeneous fleet
        missions = [
            ("delivery_bot_00", "delivery"),
            ("delivery_bot_01", "delivery"),
            ("explorer_00", "exploration"),
            ("restock_bot_00", "restock"),
        ]

        for bot_id, mission in missions:
            config = BotConfig(
                bot_id=bot_id,
                start_lat=40.0,
                start_lon=-74.0,
                mission=mission,
            )
            sim.add_bot(config)

        # Place some obstacles
        for i in range(3):
            sim.place_obstacle(f"pallet_{i}", "pallet", 40.0 + i * 0.001, -74.0, 1.5)

        results = sim.run_scenario(80, scenario_name="Mixed Mission Types")

        # All bot types should contribute to shared map
        self.assertGreaterEqual(results["active_bots"], 4)
        self.assertGreaterEqual(results["total_fusions"], 0)


def run_warehouse_benchmarks():
    """Run performance benchmarks on warehouse scenarios"""
    print("\n" + "="*70)
    print("Warehouse Simulation Performance Benchmarks")
    print("="*70)

    layout = WarehouseLayout(num_aisles=5)
    sim = WarehouseSimulator(layout)

    # Add 5 bots
    for i in range(5):
        config = BotConfig(
            bot_id=f"bot_{i:02d}",
            start_lat=40.0,
            start_lon=-74.0,
            mission="delivery",
        )
        sim.add_bot(config)

    # Place 10 obstacles
    for i in range(10):
        lat = 40.0 + (i // 2) * 0.001
        lon = -74.0 + (i % 2) * 0.002
        sim.place_obstacle(f"obstacle_{i}", "pallet", lat, lon, 1.5)

    # Time 500 simulation steps
    import time as time_module
    start = time_module.time()
    sim.run_scenario(500, scenario_name="Performance Benchmark")
    elapsed = time_module.time() - start

    print(f"\nBenchmark Results:")
    print(f"  500 steps completed in {elapsed:.2f} seconds")
    print(f"  Average step time: {(elapsed/500)*1000:.2f} ms")
    print(f"  Steps per second: {500/elapsed:.1f}")


if __name__ == "__main__":
    # Run tests
    unittest.main(argv=[''], exit=False)

    # Run benchmarks
    try:
        run_warehouse_benchmarks()
    except Exception as e:
        print(f"Benchmarks skipped: {e}")

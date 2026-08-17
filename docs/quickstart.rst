Quick Start Guide
=================

5-Minute Introduction to PyTerrainMap

Installation
------------

.. code-block:: bash

   pip install pyterrainMap

Creating Your First Map
-----------------------

.. code-block:: python

   from pyterrain_map import TerrainMap, Observation, GeoPoint

   # Create a terrain map
   terrain_map = TerrainMap()
   print(f"Created map with {len(terrain_map)} observations")
   # Output: Created map with 0 observations

Adding Observations
-------------------

Add sensor observations from robots:

.. code-block:: python

   import json
   from datetime import datetime

   # Create an observation
   obs = Observation(
       robot_id="robot-1",
       timestamp=int(datetime.now().timestamp()),
       lat=40.7128,
       lon=-74.0060,
       sensor_type="thermal",
       value_json=json.dumps({"celsius": 25.5}),
       confidence=0.95,
   )

   # Add to map
   obs_id = terrain_map.push_observation(obs)
   print(f"Added observation: {obs_id}")

Querying the Map
----------------

Query observations in a region:

.. code-block:: python

   # Query around Times Square, New York
   center = GeoPoint(40.7128, -74.0060)
   result = terrain_map.query(
       location=center,
       region_radius_km=1.0,  # 1 km radius
       time_window_seconds=86400,  # Last 24 hours
   )

   print(f"Found {result.count} observations")
   print(f"Average confidence: {result.avg_confidence:.2%}")

Analyzing Terrain
-----------------

Get terrain intelligence for a location:

.. code-block:: python

   from pyterrain_map import analyze_terrain, assess_mobility, Persona

   # Analyze terrain
   terrain = analyze_terrain(40.7128, -74.0060, radius_km=1.0)
   print(f"Summary: {terrain.summary}")
   print(f"Confidence: {terrain.confidence:.2%}")

   # Get advice for drone
   drone_advice = terrain.advice_for(Persona.Drone)
   print(f"Drone recommendations:")
   for advice in drone_advice:
       print(f"  - {advice}")

Assessing Robot Mobility
------------------------

Check if terrain is traversable for different robots:

.. code-block:: python

   # Assess mobility for different robot types
   for robot_type in ["drone", "wheeled", "quadruped", "humanoid"]:
       mobility = assess_mobility(terrain, robot_type)
       print(f"{robot_type.upper()}:")
       print(f"  Traversable: {mobility.traversable}")
       print(f"  Difficulty: {mobility.difficulty_label()}")
       print(f"  Recommended speed: {mobility.recommended_speed_ms} m/s")

Working with Multiple Robots
-----------------------------

Add observations from multiple robots:

.. code-block:: python

   import json

   # Simulate multi-robot observations at same location
   robots = ["robot-1", "robot-2", "robot-3"]
   timestamp = int(datetime.now().timestamp())

   for i, robot_id in enumerate(robots):
       obs = Observation(
           robot_id=robot_id,
           timestamp=timestamp,
           lat=40.7128 + (i * 0.001),  # Slight offset
           lon=-74.0060 + (i * 0.001),
           sensor_type="temperature",
           value_json=json.dumps({"celsius": 20.0 + i}),
           confidence=0.8 + (i * 0.05),
       )
       terrain_map.push_observation(obs)

   # Query and see consensus
   result = terrain_map.query(
       location=GeoPoint(40.7128, -74.0060),
       region_radius_km=1.0,
       time_window_seconds=3600,
   )
   print(f"Multi-robot consensus:")
   print(f"  Observations: {result.count}")
   print(f"  Average confidence: {result.avg_confidence:.2%}")

Region Statistics
-----------------

Get aggregate statistics for a region:

.. code-block:: python

   from pyterrain_map import Region

   # Define a region
   region = Region(
       north=40.8000,
       south=40.7000,
       east=-73.9000,
       west=-74.0500,
   )

   # Get statistics
   stats = terrain_map.region_stats(region)
   print(f"Region Statistics:")
   for key, value in stats.items():
       print(f"  {key}: {value}")

Understanding Data
------------------

Get metadata about available fields:

.. code-block:: python

   from pyterrain_map import explain_field

   # Learn about soil moisture
   soil_moisture_meta = explain_field("soil_moisture")
   print(f"Field: {soil_moisture_meta.field}")
   print(f"Description: {soil_moisture_meta.description}")
   print(f"Source: {soil_moisture_meta.source}")
   print(f"Normal range: {soil_moisture_meta.normal_range}")
   print(f"Confidence: {soil_moisture_meta.confidence:.2%}")

Common Patterns
---------------

**1. Real-Time Mission Planning**

.. code-block:: python

   from pyterrain_map import is_accessible

   # Quick accessibility check
   result = is_accessible(40.7128, -74.0060, "drone")
   if result["accessible"]:
       print("Mission feasible!")
       print(f"Difficulty: {result['difficulty_label']}")

**2. Multi-Robot Consensus**

.. code-block:: python

   from pyterrain_map import fuse_observations

   # Fuse observations from fleet
   observations = terrain_map.observations()
   if observations:
       fused = fuse_observations(observations[:10])
       print(f"Consensus confidence: {fused['avg_confidence']:.2%}")

**3. Terrain Change Detection**

.. code-block:: python

   from pyterrain_map import detect_changes
   from datetime import datetime, timedelta

   now = int(datetime.now().timestamp())
   yesterday = now - 86400

   changes = detect_changes(
       north=40.8, south=40.7, east=-73.9, west=-74.0,
       time_start_seconds=yesterday,
       time_end_seconds=now,
   )
   print(f"Changes detected: {changes['changes_detected']}")

Next Steps
----------

- Read :doc:`index` for a feature overview and architecture summary
- Check `GitHub Examples <https://github.com/Mullassery/PyTerrainMap/tree/main/python/examples>`_ for more tutorials
- See ``pyterrain_map.start_server()`` to run the real HTTP/HTTPS API server

Getting Help
------------

- **Documentation**: https://github.com/Mullassery/pyterrain-map
- **Issues**: https://github.com/Mullassery/pyterrain-map/issues
- **Email**: mullassery@gmail.com

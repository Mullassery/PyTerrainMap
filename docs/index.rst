PyTerrainMap Documentation
===========================

**Version:** 1.5.0

Spatial intelligence platform for multi-robot terrain mapping with temporal normalization, 3D reconstruction, and real-time fleet learning.

.. toctree::
   :maxdepth: 2
   :caption: Getting Started

   installation
   quickstart

.. toctree::
   :maxdepth: 1
   :caption: Resources

   GitHub <https://github.com/Mullassery/PyTerrainMap>
   Issues <https://github.com/Mullassery/PyTerrainMap/issues>
   Changelog <https://github.com/Mullassery/PyTerrainMap/blob/main/CHANGELOG.md>

.. note::
   This documentation currently covers installation and a quick start guide.
   Deeper architecture/API-reference pages (linked from a fuller ``toctree``
   in earlier drafts of this file) don't exist yet as standalone `.rst`
   pages -- rather than link to pages that don't exist (which breaks the
   Sphinx build), see the in-repo ``docs/*.md`` files and the Rust doc
   comments (``cargo doc --open``) for deeper reference material for now.

Quick Links
-----------

- **Installation**: :doc:`installation`
- **Quick Start**: :doc:`quickstart`

Features
--------

✨ **Multi-Robot Terrain Mapping**
   Heterogeneous robot fleets (drones, wheeled, quadrupeds, humanoids) collaboratively build shared terrain understanding.

⏱️ **Temporal Normalization**
   Time as first-class coordinate with 5D temporal metadata, handling out-of-order events and multi-clock synchronization.

🗺️ **3D Reconstruction**
   Dual-layer approach: Real-time SLAM for active localization + offline photogrammetry for persistent models.

🤖 **Traversability Intelligence**
   Terrain suitability assessment for different robot types with difficulty scoring and battery impact prediction.

🔍 **Anomaly Detection**
   8-failure taxonomy distinguishing sensor malfunction from rogue behavior, with temporal quality weighting.

💾 **Persistent World Knowledge**
   Append-only observation storage with query federation across multiple backends (SQLite, PostgreSQL, DuckDB).

🛡️ **Privacy & Security**
   RBAC with 5 roles, coordinate degradation, robot anonymization, and tamper-evident audit trails.

Installation
------------

.. code-block:: bash

   pip install pyterrainMap

Requires Python 3.10+

Quick Example
-------------

.. code-block:: python

   from pyterrain_map import TerrainMap, Observation, GeoPoint, Persona

   # Create terrain map
   map_engine = TerrainMap()

   # Add observations from robots
   obs = Observation(
       robot_id="robot-1",
       timestamp=1000,
       lat=40.7128,
       lon=-74.0060,
       sensor_type="thermal",
       value_json='{"celsius": 25.0}',
       confidence=0.95,
   )
   map_engine.push_observation(obs)

   # Analyze terrain
   from pyterrain_map import analyze_terrain, assess_mobility

   terrain = analyze_terrain(40.7128, -74.0060, radius_km=1.0)
   mobility = assess_mobility(terrain, "drone")

   print(f"Traversable: {mobility.traversable}")
   print(f"Difficulty: {mobility.difficulty_label()}")
   print(f"Recommended speed: {mobility.recommended_speed_ms} m/s")

Core Concepts
-------------

**Observation**
   Single sensor reading with spatial-temporal coordinates and confidence.

**TerrainAnalysis**
   Comprehensive analysis including observations, risks, and persona-specific recommendations.

**MobilityAssessment**
   Terrain suitability for robots with difficulty scores and battery impact.

**Temporal Metadata**
   5 dimensions: event_time, capture_time, transmission_time, ingestion_time, processing_time + clock source.

**Regional GNSS Preferences**
   Automatic selection of best GNSS source by location (NavIC in India, Galileo in Europe, etc.).

Support & Contributing
----------------------

- **GitHub**: https://github.com/Mullassery/pyterrain-map
- **Issues**: https://github.com/Mullassery/pyterrain-map/issues
- **Email**: mullassery@gmail.com

License
-------

Proprietary License -- free to use with explicit attribution.
Copyright © 2026 Georgi Mammen Mullassery. See `LICENSE
<https://github.com/Mullassery/PyTerrainMap/blob/main/LICENSE>`_ for the
full terms.

Indices and tables
==================

* :ref:`genindex`
* :ref:`modindex`
* :ref:`search`

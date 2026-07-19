#!/usr/bin/env python3
"""Basic PyTerrainMap terrain analysis example."""

from pyterrain_map import Persona

def main():
    """Analyze location for mobile robot."""

    # Coordinates: Bangalore, India
    latitude = 12.9716
    longitude = 77.5946

    print(f"Analyzing terrain at {latitude}, {longitude}")
    print(f"Available personas: {list(Persona.values())}")

    # Would analyze here once full API is bound
    # analysis = engine.analyze(latitude, longitude, Persona.MobileRobot)
    # print(f"Summary: {analysis.summary}")
    # print(f"Risks: {[r.severity_label() for r in analysis.risks]}")


if __name__ == "__main__":
    main()

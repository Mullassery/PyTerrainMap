# PyTerrainMap Python Bindings Setup

## Overview

PyTerrainMap exposes its high-performance Rust core to Python via PyO3 native extension module (abi3-py310). This enables both:

1. **Python Packages**: `pip install pyterrain-map` (when published)
2. **Python API**: `from pyterrain_map import ...`
3. **CLI Tool**: `pytm <command>`

## Architecture

### Build Stack
```
Python 3.13 (system)
    ↓
PyO3 0.22 (extension-module)
    ↓
Rust Core (pyterrain-map crate)
    ↓
maturin (Python build backend)
    ↓
Wheel (pyterrain_map-0.0.1-cp310-abi3-macosx_11_0_arm64.whl)
    ↓
pip/uv install -e .
```

### ABI3 Stable Binary Interface

The build uses `abi3-py310` feature, which creates a **stable ABI wheel** compatible with:
- Python 3.10, 3.11, 3.12, 3.13+ (forward compatible)
- Works across minor Python versions without recompilation
- Single wheel file for broad distribution

### Files Structure

```
pyterrain-map/
├── Cargo.toml                          # Rust package config + PyO3 dependency
├── pyproject.toml                      # Python package config + maturin backend
├── src/
│   ├── lib.rs                         # Rust library root (includes py module)
│   ├── py.rs                          # PyO3 #[pymodule] entry point
│   └── ...                            # All 25+ Rust modules (unchanged)
├── python/
│   └── pyterrain_map/
│       ├── __init__.py                # Python wrapper module
│       ├── cli.py                     # CLI entry point
│       └── types.py                   # (optional) Python-only types
├── python/
│   └── examples/
│       └── basic_analysis.py          # Example usage
└── PYPROJECT_SETUP.md                 # This file
```

## Python Bindings Roadmap

### Phase 1: Foundation (Current - Week 19)
- ✅ PyO3 module structure created
- ✅ Cargo.toml: PyO3 0.22 + abi3-py310 enabled
- ✅ pyproject.toml: maturin build backend configured
- ✅ src/py.rs: Module entry point with Persona enum
- ✅ python/pyterrain_map/__init__.py: High-level Python wrapper types
- ✅ python/pyterrain_map/cli.py: CLI entry point
- ✅ Successful build and install with maturin develop

### Phase 2: Core Type Bindings (Week 20)
**Expose Rust types as Python classes via PyO3:**

1. **Intelligence Layer** (#[pyclass] + #[pymethods])
   - `TerrainAnalysis` → Python class with properties/methods
   - `MobilityAssessment` → Robot traversability results
   - `EnvironmentalConditions` → Weather + soil integration
   - `Risk` → Risk assessment with severity_label()
   - `DataExplanation` → Self-documenting fields

2. **Spatial Reasoning** (#[pyclass] + #[pymethods])
   - `SpatialReasoningEngine` → Multi-source aggregation
   - `DataProvenance` → Source attribution
   - `Uncertainty` → Confidence modeling
   - `PositionAnswer` → Position with provenance

3. **Query & Fusion**
   - `ObservationStore` → Append-only storage interface
   - `SensorFusion` → Multi-sensor fusion results
   - `Query` → Spatial/temporal querying

### Phase 3: Full API Surface (Week 21)
- Implement PyO3 From/Into traits for seamless Rust↔Python conversion
- Expose all public types from Rust modules
- Create comprehensive docstrings for auto-generated docs
- Python type hints (.pyi stubs)

### Phase 4: Performance & Production (Week 22+)
- Benchmark Python API performance (target <5ms analysis)
- Profile memory usage with large datasets
- Optimize hot paths if needed
- Create performance documentation

## Building from Source

### Prerequisites
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Ensure Python 3.10+ is available
python --version  # Should show 3.10+

# Install maturin
pip install maturin
```

### Development Build
```bash
cd pyterrain-map
maturin develop              # Builds wheel and installs editable
# or
pip install -e .            # Also works with maturin backend
```

### Release Build
```bash
maturin build --release      # Optimized wheel in ./dist/
pip install dist/pyterrain_map-*.whl
```

### Testing
```bash
python -c "import pyterrain_map; print(pyterrain_map.__version__)"
pytm help
```

## Implementation Notes

### PyO3 0.22 Features Used
- `#[pymodule]` — Module entry point
- `#[pyclass]` — Expose Rust struct as Python class
- `#[pymethods]` — Implement Python methods
- `#[pyo3(annotation)]` — Python type hints
- ABI3 stable interface — Forward-compatible wheels

### Python Type Bindings Pattern

```rust
// Rust side (src/py_bindings.rs - to be created)
#[pyclass]
pub struct TerrainAnalysis {
    #[pyo3(get)]
    pub location: (f64, f64),
    #[pyo3(get)]
    pub summary: String,
}

#[pymethods]
impl TerrainAnalysis {
    #[pyo3(name = "severity_label")]
    fn severity_label(&self) -> String { /* ... */ }
}
```

```python
# Python side (usage)
from pyterrain_map import TerrainAnalysis
analysis = TerrainAnalysis(...)
print(analysis.summary)           # Property access
print(analysis.severity_label()) # Method call
```

## Integration with AI Agents

### Claude Code / MCP Integration
```python
from pyterrain_map import TerrainMap, Persona, MCPTool

# Expose as tools to Claude Code
tools = [
    MCPTool.terrain_assessment(),
    MCPTool.mobility_assessment(),
    MCPTool.explain_field(),
]

# Claude Code can discover and call PyTerrainMap
```

### Autonomous Decision-Making
```python
from pyterrain_map import TerrainMap, Persona

engine = TerrainMap()

# Analyze location for robot decision
analysis = engine.analyze(lat, lon, Persona.MobileRobot)

# Use results for autonomous planning
if analysis.is_traversable and analysis.confidence > 0.8:
    plan_route()
else:
    seek_alternative()
```

## Performance Targets

Based on Rust core performance:

| Operation | Target | Notes |
|-----------|--------|-------|
| Terrain analysis | <5ms | Per location |
| Mobility assessment | <2ms | Per location |
| Batch (1000 locations) | <2 sec | ~2ms each |
| Spatial query (1M obs) | <50ms | H3 indexed |
| Temporal query | <20ms | Binary search |
| Anomaly detection | <10ms | Per observation |

Python overhead expected ~10-20% vs raw Rust.

## Known Limitations & Roadmap

### Python 3.13 Compatibility
- ✅ Enabled via abi3-py310 stable ABI
- No special handling needed for Python 3.13+
- Wheel forward-compatible

### Async Support
- PyO3 async not yet enabled (pyo3-asyncio deferred)
- Sync API currently available
- Async API (Week 22+) via async-generator pattern

### Features Deferred
- [ ] Full PyO3 type wrapping (Phase 2)
- [ ] Detailed docstrings (Phase 2)
- [ ] Type stubs (.pyi) (Phase 3)
- [ ] Performance benchmarks (Phase 4)
- [ ] Async API (Week 22+)

## Next Steps

1. **Week 20**: Implement Phase 2 type bindings
2. **Week 21**: Complete full API surface + docs
3. **Week 22**: Performance optimization + production hardening
4. **Week 23+**: PyPI publication

## References

- [PyO3 Documentation](https://pyo3.rs/)
- [maturin Guide](https://www.maturin.rs/)
- [Stable ABI](https://pyo3.rs/latest/faq#how-can-i-create-a-stable-abi-across-python-versions)
- [PYTHON_BINDINGS.md](./PYTHON_BINDINGS.md) — High-level Python API docs

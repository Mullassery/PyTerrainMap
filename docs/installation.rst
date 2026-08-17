Installation
=============

Requirements
------------

- Python 3.10 or newer
- pip or uv package manager
- ~100MB disk space

Installation from PyPI
----------------------

The easiest way to install PyTerrainMap is from PyPI:

.. code-block:: bash

   pip install pyterrainMap

For uv users:

.. code-block:: bash

   uv pip install pyterrainMap

Verify Installation
--------------------

Test the installation by importing PyTerrainMap:

.. code-block:: python

   import pyterrain_map
   print(pyterrain_map.__version__)
   # Output: the installed version, e.g. 1.5.0

Platform-Specific Notes
-----------------------

**macOS (Apple Silicon - M1/M2/M3)**

Native arm64 wheels are provided:

.. code-block:: bash

   pip install pyterrainMap  # Auto-selects arm64 wheel

**macOS (Intel x86_64)**

x86_64 wheels are available:

.. code-block:: bash

   pip install pyterrainMap  # Auto-selects x86_64 wheel

**Linux (x86_64)**

Linux wheels support glibc 2.31+:

.. code-block:: bash

   pip install pyterrainMap

**Windows (x86_64)**

Windows wheels for Python 3.10-3.13:

.. code-block:: bash

   pip install pyterrainMap

Optional Dependencies
---------------------

For development and testing:

.. code-block:: bash

   pip install pyterrainMap[dev]

For documentation building:

.. code-block:: bash

   pip install pyterrainMap[docs]

For machine learning features:

.. code-block:: bash

   pip install pyterrainMap[scientific]

For database backend support:

.. code-block:: bash

   pip install pyterrainMap[database]

For image processing (georeferencing):

.. code-block:: bash

   pip install pyterrainMap[imaging]

Install All Extras
-------------------

.. code-block:: bash

   pip install pyterrainMap[dev,docs,scientific,database,imaging]

Building from Source
---------------------

For development or custom builds:

.. code-block:: bash

   git clone https://github.com/Mullassery/pyterrain-map.git
   cd pyterrain-map
   pip install maturin
   maturin develop

Troubleshooting
---------------

**ImportError: No module named 'pyterrain_map'**

Ensure installation completed successfully:

.. code-block:: bash

   pip install --force-reinstall pyterrainMap

**Wheel compatibility error**

Upgrade pip and check your Python version:

.. code-block:: bash

   python --version  # Should be 3.10+
   pip install --upgrade pip

**macOS M1/M2 arm64 issues**

Ensure you're using native Python (not Rosetta):

.. code-block:: bash

   python -c "import platform; print(platform.machine())"
   # Should output: arm64

**Linux glibc version mismatch**

Check your glibc version:

.. code-block:: bash

   ldd --version | head -n1

Need at least glibc 2.31. For older systems, build from source.

Getting Help
------------

- **Documentation**: https://github.com/Mullassery/pyterrain-map
- **Issues**: https://github.com/Mullassery/pyterrain-map/issues
- **Email**: mullassery@gmail.com

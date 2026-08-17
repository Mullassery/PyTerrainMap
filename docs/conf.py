"""Sphinx documentation configuration for PyTerrainMap."""

import os
import sys
from pathlib import Path

# Add source to path
sys.path.insert(0, str(Path(__file__).parent.parent))

project = "PyTerrainMap"
copyright = "2026, Georgi Mammen Mullassery"
author = "Georgi Mammen Mullassery"
version = "1.5.0"
release = "1.5.0"

# Extensions
extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.autosummary",
    "sphinx.ext.intersphinx",
    "sphinx.ext.viewcode",
    "sphinx_rtd_theme",
    "sphinx.ext.mathjax",
    "sphinx_autodoc_typehints",
]

# Templates
templates_path = ["_templates"]
exclude_patterns = ["_build", "Thumbs.db", ".DS_Store"]

# HTML output
html_theme = "sphinx_rtd_theme"
html_theme_options = {
    "logo_only": False,
    "display_version": True,
    "prev_next_buttons_location": "bottom",
    "style_external_links": False,
    "vcs_pageview_mode": "",
    "style_nav_header_background": "#2c3e50",
}
html_static_path = ["_static"]
html_css_files = ["custom.css"]

# Autodoc
autodoc_default_options = {
    "members": True,
    "member-order": "bysource",
    "undoc-members": False,
    "show-inheritance": True,
}

# Intersphinx
intersphinx_mapping = {
    "python": ("https://docs.python.org/3", None),
    "numpy": ("https://numpy.org/doc/stable", None),
    "pydantic": ("https://docs.pydantic.dev/latest", None),
}

# Source file suffix and master doc
source_suffix = ".rst"
master_doc = "index"

# LaTeX output
latex_elements = {
    "papersize": "letterpaper",
    "pointsize": "11pt",
    "preamble": r"\usepackage{charter}",
    "fncychap": "\\usepackage[Bjornstrup]{fncychap}",
}

latex_documents = [
    ("index", "pyterrain_map.tex", "PyTerrainMap Documentation", author, "manual"),
]

# Viewcode
viewcode_import = False

# Typehints
autodoc_typehints = "description"
typehints_defaults = "comma"

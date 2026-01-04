# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

import os
import sys
from pathlib import Path

# Add the project root to the Python path for imports
project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root))

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = "tarzi"
copyright = "2025-2026, Mirasoth Inc."
author = "Mirasoth Team"
release = "latest"

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.autosummary",
    "sphinx.ext.viewcode",
    "sphinx.ext.napoleon",
    "sphinx.ext.intersphinx",
    "sphinx.ext.todo",
    "sphinx.ext.coverage",
    "sphinx.ext.ifconfig",
    "sphinx_autodoc_typehints",
    "sphinx_copybutton",
    "myst_parser",
    "sphinx_tabs.tabs",
    "sphinx_design",
]

# Add any paths that contain templates here, relative to this directory.
templates_path = ["_templates"]

# List of patterns, relative to source directory, that match files and
# directories to ignore when looking for source files.
exclude_patterns = ["_build", "Thumbs.db", ".DS_Store", "README.md"]

# The suffix(es) of source filenames.
source_suffix = {
    ".rst": None,
    ".md": "markdown",
}

# The master toctree document.
master_doc = "index"

# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = "furo"
html_static_path = ["_static"]

# Custom CSS and JavaScript
html_css_files = [
    "custom.css",
]

# Theme options
html_theme_options = {
    "sidebar_hide_name": False,
    "light_logo": "tarzi-logo-light.png",
    "dark_logo": "tarzi-logo-dark.png",
    "source_repository": "https://github.com/mirasoth/tarzi/",
    "source_branch": "main",
    "source_directory": "docs/",
}

# The name of an image file (relative to this directory) to place at the top
# of the sidebar.
# html_logo = "_static/tarzi-logo.png"  # Commented out - using Furo theme logos instead

# The name of an image file (within the static path) to use as favicon of the
# docs.
html_favicon = "_static/favicon.ico"

# -- Extension configuration -------------------------------------------------

# -- Options for autodoc extension -------------------------------------------
autodoc_default_options = {
    "members": True,
    "undoc-members": True,
    "exclude-members": "__weakref__",
}

# -- Options for autosummary extension ---------------------------------------
autosummary_generate = True

# -- Options for napoleon extension ------------------------------------------
napoleon_include_special_with_doc = True
napoleon_use_param = True
napoleon_use_rtype = True

# -- Options for intersphinx extension ---------------------------------------
intersphinx_mapping = {
    "python": ("https://docs.python.org/3", None),
    "requests": ("https://requests.readthedocs.io/en/stable/", None),
}

# -- Options for autodoc_typehints extension ---------------------------------
always_document_param_types = True
typehints_document_rtype = True

# -- Options for myst-parser extension ---------------------------------------
myst_enable_extensions = [
    "colon_fence",
    "deflist",
    "dollarmath",
    "fieldlist",
    "html_admonition",
    "html_image",
    "replacements",
    "smartquotes",
    "strikethrough",
    "substitution",
    "tasklist",
]

# -- Options for copybutton extension ----------------------------------------
copybutton_prompt_text = r">>> |\.\.\. |\$ |In \[\d*\]: | {2,5}\.\.\.: | {5,8}: "
copybutton_prompt_is_regexp = True
copybutton_line_continuation_character = "\\"

# -- Options for todo extension ----------------------------------------------
todo_include_todos = True

# -- Custom setup ------------------------------------------------------------
def setup(app):
    """Custom setup function for Sphinx."""
    app.add_css_file("custom.css")
    
    # Add custom roles and directives if needed
    pass 
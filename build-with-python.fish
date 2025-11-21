#!/usr/bin/env fish

# Helper script to build tcp-lab with Python support
# This script ensures PyO3 uses the correct Python interpreter from uv

# Get the Python interpreter path from uv
set python_path (cd sdk/python && uv run python -c "import sys; print(sys.executable)")

# Export PYO3_PYTHON for the build
set -x PYO3_PYTHON $python_path

echo "Using Python: $python_path"

# Run cargo with all provided arguments
cargo $argv

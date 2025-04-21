#!/bin/bash
# Load environment variables from .env.python and export them
set -a
source .env.python
set +a

# Print environment for debugging
echo "Using PYO3_PYTHON: $PYO3_PYTHON"
echo "Using LD_LIBRARY_PATH: $LD_LIBRARY_PATH"

# Verify Python library exists
ls -la $LD_LIBRARY_PATH/libpython*.so* || echo "Python library not found in $LD_LIBRARY_PATH"

# Run the specified test with environment variables set
cargo test test_python_skyfield_available -- --nocapture
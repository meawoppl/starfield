#!/bin/bash
set -e

# Configuration variables
PYENV_ROOT="$HOME/.pyenv"
PYTHON_VERSION="3.10.8"
PYENV_ENV_NAME="starfield-env"

echo "=== Setting up Python environment for Starfield comparison tests ==="

# Check if pyenv is installed
if ! command -v pyenv &> /dev/null; then
    echo "Pyenv not found. Installing pyenv..."
    curl https://pyenv.run | bash
    
    # Add pyenv to PATH for current session
    export PATH="$PYENV_ROOT/bin:$PATH"
    eval "$(pyenv init --path)"
    eval "$(pyenv init -)"
else
    echo "Pyenv already installed."
fi

# Install Python 3.11 via pyenv if not already installed
if ! pyenv versions | grep -q "$PYTHON_VERSION"; then
    echo "Installing Python $PYTHON_VERSION..."
    pyenv install $PYTHON_VERSION
else
    echo "Python $PYTHON_VERSION already installed."
fi

# Create or update pyenv environment
if ! pyenv versions | grep -q "$PYENV_ENV_NAME"; then
    echo "Creating pyenv environment '$PYENV_ENV_NAME'..."
    pyenv virtualenv $PYTHON_VERSION $PYENV_ENV_NAME
else
    echo "Pyenv environment '$PYENV_ENV_NAME' already exists."
fi

# Set local Python version to our environment
echo "Setting '$PYENV_ENV_NAME' as local Python environment..."
pyenv local $PYENV_ENV_NAME

# Install required packages
echo "Installing required Python packages..."
pip install --upgrade pip
pip install skyfield pytest

# Create a small test script to verify skyfield installation
TEST_SCRIPT=$(cat <<EOF
#!/usr/bin/env python

try:
    import skyfield
    from skyfield.api import load
    from skyfield.data import hipparcos
    
    print(f"Skyfield {skyfield.__version__} installed successfully!")
    
    # Test loading hipparcos catalog
    print("Testing Hipparcos catalog access...")
    from skyfield.api import Star
    # Create a simple Sirius star to verify Star object works
    sirius = Star(ra_hours=6.75, dec_degrees=-16.7)
    print(f"Created Star object for Sirius: {sirius}")
    
    # Just demonstrate that the hipparcos module is available
    print(f"Hipparcos module available: {hipparcos.__name__}")
    
except Exception as e:
    print(f"Error: {e}")
    exit(1)

print("Skyfield is working correctly!")
EOF
)

echo "$TEST_SCRIPT" > test_skyfield.py
chmod +x test_skyfield.py

echo "Running Skyfield test script..."
./test_skyfield.py

echo ""
echo "=== Python environment setup complete ==="
echo ""
echo "The pyenv environment '$PYENV_ENV_NAME' is now configured with Python $PYTHON_VERSION"
echo "It is set as the local Python version for this directory."
echo ""
echo "To run Rust tests that use Python comparison:"
echo "  cargo test python_comparison"
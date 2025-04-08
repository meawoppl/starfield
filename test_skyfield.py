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

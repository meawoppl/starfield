# JPL Ephemeris (jplephem) Rust Port

This document outlines the plan for porting the Python jplephem library to Rust as a submodule of the Starfield project.

## Overview
The jplephem library provides functionality for reading and interpreting JPL Development Ephemerides (DE) files, which contain high-precision position and velocity data for solar system bodies. These are distributed as binary SPK (Spacecraft Planet Kernel) files in the SPICE format.

## Library Structure and Functionality

The Python jplephem library consists of the following main components:

1. **DAF (Double Array File) Module**
   - Low-level handling of the binary file format that underlies SPK files
   - Memory mapping for efficient file access
   - Reading and parsing of file summaries and data segments

2. **SPK (Spacecraft Planet Kernel) Module**
   - High-level interface for accessing planetary position data
   - Support for different SPK segment types (2, 3, and 9)
   - Chebyshev polynomial interpolation for precise positions

3. **PCK (Planetary Constants Kernel) Module**
   - Support for rotation data of celestial bodies
   - Similar to SPK but focuses on orientation rather than position

4. **Supporting Modules**
   - Calendar/time conversions (Julian dates)
   - Name/ID mappings for celestial bodies
   - Error handling
   - Command-line utilities

## Plan of Action

### Phase 1: Project Setup
- [x] Create basic module structure in src/jplephem/
- [x] Set up module declarations and add to Cargo.toml
- [x] Define error types with thiserror
- [x] Add test data files (copy from python-jplephem/ci/)

### Phase 2: DAF Implementation
- [x] Implement basic file and record handling
- [x] Add memory mapping support with memmap2
- [x] Implement methods for reading summaries
- [x] Support big and little endian format detection
- [x] Add support for reading arrays and segments

### Phase 3: SPK Implementation
- [ ] Create SPK and Segment structs
- [ ] Implement segment access with center/target indexing
- [ ] Add support for Type 2 segments (position only)
- [ ] Add support for Type 3 segments (position and velocity)
- [ ] Add support for Type 9 segments (discrete points)
- [ ] Implement Chebyshev polynomial interpolation

### Phase 4: Time Handling and Utilities
- [ ] Integrate with existing Starfield time module
- [ ] Implement Julian date conversions (porting calendar.py)
- [ ] Add celestial body name/ID mappings (port names.py)
- [ ] Implement PCK file support for rotation data

### Phase 5: Testing and Validation
- [ ] Create unit tests for DAF functionality
- [ ] Add tests for SPK segment handling
- [ ] Test Chebyshev polynomial interpolation
- [ ] Add integration tests comparing outputs with Python implementation
- [ ] Validate against DE405, DE421, DE430 test data
- [ ] Benchmark performance vs. Python implementation

### Phase 6: API Refinement
- [ ] Design idiomatic Rust API that aligns with Starfield conventions
- [ ] Implement traits for common operations
- [ ] Add support for vectorized calculations with nalgebra
- [ ] Ensure proper error handling and result propagation
- [ ] Add comprehensive documentation with examples

### Phase 7: Example Implementations
- [ ] Create basic example showing planetary positions
- [ ] Add example comparing outputs with Python implementation
- [ ] Create visualization example if applicable

## Completed Steps

1. Created the basic module structure in src/jplephem/
2. Set up module declarations and added required dependencies to Cargo.toml
3. Created error types using thiserror
4. Created skeleton code for all modules
5. Added test data files from the Python jplephem repository
6. Implemented DAF (Double Array File) reader:
   - Support for reading binary DAF files both big and little endian
   - Memory mapping for efficient access to large files
   - Reading file headers, comments, and summaries
   - Extracting arrays of numerical data
   - Successfully tested with DE421 and PCK test files
   - Fixed compatibility issues with different SPK/PCK file formats

## Dependencies
- **nalgebra**: For vector/matrix operations
- **memmap2**: For memory-mapped file access
- **byteorder**: For handling endianness in binary file format
- **thiserror**: For error handling
- **num-traits**: For numeric trait implementations
- **lazy_static**: For celestial body name/ID mappings

## Implementation Notes

### Memory Management
- Use memory mapping for efficient handling of large ephemeris files
- Support fallback to regular file I/O when memory mapping isn't available

### Performance Considerations
- Optimize Chebyshev polynomial evaluation for speed
- Use SIMD operations where possible
- Consider pre-computing coefficients for hot paths

### Safety and Error Handling
- Provide detailed error messages with context
- Handle out-of-range dates with specific error type
- Implement proper resource cleanup with Drop trait
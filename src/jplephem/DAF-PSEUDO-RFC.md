```
          DAF - Double Precision Array Files
          
                  Format Specification
                        
                      Version 1.0
```

# 1. Introduction

## 1.1 Purpose

This document specifies the Double Precision Array File (DAF) format used by the NASA SPICE system for storing and retrieving double precision arrays across computing platforms. DAF files form the foundation for several SPICE file formats, including SPK (Spacecraft and Planet Kernel) and PCK (Planetary Constants Kernel) files.

## 1.2 Scope

This specification defines the structure, organization, and constraints of the DAF format, with the goal of providing a common reference for developers implementing DAF readers and writers.

## 1.3 Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://tools.ietf.org/html/rfc2119).

# 2. File Architecture

## 2.1 General Structure

A DAF file is a structured, direct-access binary file consisting of fixed-length records of 1024 bytes each. Records are numbered consecutively, beginning with 1. The file uses a flat, direct-access structure originally designed for Fortran-77 direct access files.

## 2.2 Data Type Assumptions

The DAF format assumes:
- 1-byte character values
- 4-byte integer values
- 8-byte double precision values

## 2.3 Endianness

DAF files MAY be stored in either big-endian ("BIG-IEEE") or little-endian ("LTL-IEEE") format. The format is indicated in the file record.

# 3. Record Types

A DAF file consists of five types of records:

1. File Record - The first record (record 1) containing global metadata
2. Reserved Records - Optional records for storing human-readable comments
3. Summary Records - Metadata records describing the arrays contained in the file
4. Name Records - Records containing array names corresponding to summaries
5. Element Records - Records containing the actual double precision array data

# 4. File Record Format

The file record MUST be the first record in a DAF file and contains the following fields:

| Offset | Size (bytes) | Type   | Name      | Description                               |
|--------|--------------|--------|-----------|-------------------------------------------|
| 0      | 8            | char   | LOCIDW    | Identification string ("NAIF/DAF" or "DAF/xyz") |
| 8      | 4            | int    | ND        | Number of double precision components in each array summary |
| 12     | 4            | int    | NI        | Number of integer components in each array summary |
| 16     | 60           | char   | LOCIFN    | Internal file name                         |
| 76     | 4            | int    | FWARD     | Record number of first summary record      |
| 80     | 4            | int    | BWARD     | Record number of last summary record       |
| 84     | 4            | int    | FREE      | Record number of first free record         |
| 88     | 8            | char   | LOCFMT    | Binary format indicator (BIG-IEEE/LTL-IEEE) |
| 96     | 928          | -      | FTPSTR    | Reserved/FTP validation string (optional)  |

## 4.1 Constraints

The following constraints apply to the file record:
- ND MUST be between 0 and 124, inclusive
- NI MUST be between 2 and 250, inclusive
- FWARD MUST be a positive integer
- BWARD MUST be a positive integer
- FREE MUST be a positive integer

# 5. Comment Area

## 5.1 Structure

If present, the comment area begins at record 2 and continues up to but not including the record indicated by FWARD. Comments are stored as ASCII text, with NULL bytes (0x00) used as line separators. The comment area MUST be terminated by an EOT byte (0x04).

# 6. Summary and Name Records

## 6.1 Summary Record Structure

Each summary record contains:
- A forward pointer (8 bytes) to the next summary record
- A backward pointer (8 bytes) to the previous summary record
- A count (8 bytes) of the number of summaries in the record
- A sequence of array summaries

The count of summaries is stored as a double precision value to maintain alignment.

## 6.2 Array Summary Format

Each array summary consists of:
- ND double precision components (8 bytes each)
- NI integer components (4 bytes each, stored as double precision)

The first two integer components MUST be:
1. Initial address of the array elements in the file
2. Final address of the array elements in the file

## 6.3 Name Records

For each summary record, a corresponding name record contains the names of the arrays described in the summary record. Each name has the same length as the corresponding summary.

# 7. Array Storage

## 7.1 Organization

Array elements are stored contiguously in element records, with no padding or special alignment requirements. Arrays are identified by their initial and final addresses in the file, which are stored in the array summary.

## 7.2 Addressing

Array addresses in DAF files are 1-based, meaning the first element of an array has address 1. Addresses refer to double precision values, not bytes.

## 7.3 Data Storage

Elements are stored in a flat array format. Any logical structuring of the data (e.g., into matrices) MUST be handled by the application reading the file, based on metadata in the array summary.

# 8. Implementation Notes

## 8.1 Reading DAF Files

Implementations SHOULD:
1. Verify the file identification string
2. Determine endianness from the format string
3. Read the file record to obtain global parameters
4. Traverse the summary records as a doubly-linked list
5. Use the array addresses to access array data

## 8.2 Writing DAF Files

When creating DAF files, implementations MUST:
1. Create a valid file record with appropriate parameters
2. Maintain consistent endianness throughout the file
3. Update forward and backward pointers in summary records
4. Ensure array summaries contain valid initial and final addresses

## 8.3 Cross-Platform Concerns

Since DAF files can be transferred between different computing platforms, implementations SHOULD handle byte-swapping as needed based on the format indicator in the file record.

# 9. References

1. NAIF/JPL. "DAF Required Reading." NASA Jet Propulsion Laboratory. https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/daf.html

# Appendix A. Example Summary Record Layout

For a DAF file with ND=2 and NI=6, each array summary contains:
- 2 double precision values (16 bytes)
- 6 integer values stored as double precision (48 bytes)

The total size of each summary is 64 bytes.

If a summary record has room for 15 summaries:
- The first 24 bytes contain control information
- The remaining 1000 bytes contain 15 summaries of 64 bytes each = 960 bytes
- 40 bytes of padding at the end of the record

# Appendix B. Array Addressing Example

For an array with initial address 1000 and final address 1999:
- The array contains 1000 double precision elements
- Elements are stored at addresses 1000 through 1999
- The array requires 8000 bytes of storage (1000 elements × 8 bytes/element)
- Elements would be stored across multiple records as needed
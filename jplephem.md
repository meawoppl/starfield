# **Technical Specification of the JPL SPICE Kernel (SPK) Binary File Format (.bsp) and Underlying DAF Architecture**

## **1\. Introduction**

### **1.1. Purpose and Scope**

This report provides an expert-level technical description of the Jet Propulsion Laboratory (JPL) SPICE Kernel (SPK) binary file format, commonly identified by the .bsp extension.1 It delves into the structure, data encoding, content boundaries, and data extraction mechanisms associated with these files. Central to this description is the underlying Double precision Array File (DAF) architecture, which forms the foundation for binary SPK files. The scope of this document is focused on the technical specification of the binary file format itself, clarifying what information is contained within .bsp files and how it is organized at a low level. It does not provide exhaustive tutorials on SPICE Toolkit API usage or detailed mathematical derivations for all possible ephemeris data types, but rather aims to equip developers and researchers with a precise understanding of the file structure for purposes of implementation, validation, or data analysis.

### **1.2. The SPICE Ecosystem Context**

The SPK format exists within the broader context of the Spacecraft, Planet, Instrument, C-matrix, Events (SPICE) system, developed and maintained by the Navigation and Ancillary Information Facility (NAIF) team at JPL.2 SPICE provides a unified environment for scientists and engineers to access and utilize ancillary data necessary for space mission design, operations, and data analysis.6 SPK files represent just one component, or "kernel," within this system.2 Kernels are essentially files containing specific types of ancillary data.3

SPK files are specifically designed to store ephemeris data – typically Cartesian state vectors comprising position and velocity – for objects such as spacecraft, planets, natural satellites, comets, and asteroids.1 These state vectors describe the object's motion relative to a specified center within a defined reference frame over a particular time span.1

To perform comprehensive geometric calculations, SPK files are almost always used in conjunction with other SPICE kernel types. These include:

* **CK (C-matrix Kernel):** Provides orientation or attitude data for spacecraft, instruments, or other structures.3  
* **PCK (Planetary Constants Kernel):** Contains physical constants, size, shape, and orientation information for celestial bodies.3  
* **LSK (Leapseconds Kernel):** Defines the relationship between Coordinated Universal Time (UTC) and Ephemeris Time (ET), also known as Barycentric Dynamical Time (TDB).2  
* **FK (Frames Kernel):** Specifies the relationships between different reference frames used within SPICE.3  
* **IK (Instrument Kernel):** Contains parameters describing the geometry and operational characteristics of scientific instruments.3  
* **SCLK (Spacecraft Clock Kernel):** Relates onboard spacecraft clock time to ET.3

Understanding the distinct roles of these kernel types is crucial for appreciating the scope of data contained within an SPK file, as detailed further in Section 4\.

### **1.3. Foundational Technologies: DAF and Access Tools**

Binary SPK files, along with binary CK and PCK files, are built upon the Double precision Array File (DAF) architecture.1 DAF is a low-level file structure developed by NAIF specifically designed for the efficient storage and direct access of large arrays of double-precision floating-point numbers.14 It provides the underlying mechanism for organizing and retrieving the numerical data (e.g., polynomial coefficients, discrete states) that constitute the ephemeris information within an SPK file.9

Due to their binary nature and specialized internal structure, DAF-based files like SPK kernels cannot be readily interpreted by standard text editors or generic data analysis tools.3 Interaction with these files necessitates the use of dedicated software libraries. The primary tool is the official NASA SPICE Toolkit, available in Fortran (SPICELIB), C (CSPICE), IDL (icy), and MATLAB (mice) versions.5 Additionally, third-party libraries have been developed to provide access within other programming environments, a notable example being python-jplephem for Python, which focuses specifically on reading SPK files.17 These tools abstract the complexities of the DAF structure and the various SPK data types, providing higher-level interfaces for data retrieval.

## **2\. The DAF (Double precision Array File) Architecture**

### **2.1. Core Concepts and Design Philosophy**

DAF, an acronym for "Double precision Array File," represents a file architecture engineered by NAIF to efficiently manage large datasets composed primarily of numerical data.14 Its design aims to merge the benefits of contiguous array storage with the flexibility of direct access files, while mitigating the drawbacks traditionally associated with each approach.14 A core principle is the storage of "pure" double precision numbers, ensuring data integrity and facilitating numerical computations.14

A significant design goal for DAF was portability across diverse computing environments.14 In the historical context of scientific computing, variations in hardware architectures, particularly concerning byte order (endianness), posed challenges for data exchange. The DAF specification addresses this by incorporating mechanisms to identify the binary format used to write the file. The SPICE Toolkit includes capabilities for run-time binary file format translation, allowing DAF-based files created on one architecture (e.g., big-endian) to be read correctly on another (e.g., little-endian).9 While this translation is often handled transparently by modern SPICE library functions, the underlying format distinction, explicitly stored within the file header (see Section 2.3), remains a critical piece of information. It is essential for the correct functioning of non-SPICE tools attempting to parse DAF files and for diagnosing potential file corruption issues that might arise from improper file transfer methods (e.g., FTP in ASCII mode instead of binary mode).14 Historically, a text-based "transfer format" (often with a .xsp extension for SPK files) was used for porting binary kernels, but this is now largely superseded by the built-in translation capabilities.1

### **2.2. File Structure Overview**

Conceptually, a DAF file can be viewed as a linear sequence of numbered storage slots called "words".14 Each word is sized to hold exactly one double-precision floating-point number, which typically corresponds to 8 bytes on supported platforms.14 These words are organized into fixed-size physical records. The standard record length for DAF files is 1024 bytes, meaning each physical record contains 128 double-precision words.14

A DAF file is composed of five distinct types of physical records 14:

1. **File Record:** A single record, always located at the beginning of the file (Record 1), containing global metadata about the DAF file itself.  
2. **Reserved Records (Comment Area):** An optional, contiguous block of records immediately following the File Record, reserved for storing textual comments or annotations.  
3. **Summary Records:** Records containing metadata ("summaries" or "descriptors") for the data arrays stored in the file, along with pointers that link these records into a doubly-linked list.  
4. **Name Records:** Records containing character string names associated with the data arrays. Each Name Record corresponds to a specific Summary Record.  
5. **Element Records:** The most numerous record type, containing the actual double-precision numerical data elements that make up the arrays stored within the DAF.

### **2.3. The File Record (Record 1\)**

The File Record is invariably the first physical record (1024 bytes) in any DAF file and serves as the primary header, containing essential information for interpreting the rest of the file.14 Its structure is fixed and contains several critical fields.

**Table 1: DAF File Record Layout (Bytes 0-1023)**

| Offset (Bytes) | Field Name | Data Type | Size (Bytes) | Description |
| :---- | :---- | :---- | :---- | :---- |
| 0 | LOCIDW | char | 8 | Location Identification Word. Typically 'DAF/xxxx' (e.g., 'DAF/SPK'). Used to verify the file is a DAF and indicate the data type.14 |
| 8 | ND | int | 4 | Number of double precision components per array summary.14 |
| 12 | NI | int | 4 | Number of integer components per array summary.14 |
| 16 | LOCIFN | char | 60 | Location Internal File Name. A descriptive name or label for the file, stored internally.14 |
| 76 | FWARD | int | 4 | Forward Record Pointer. Record number of the first (initial) summary record in the file.14 |
| 80 | BWARD | int | 4 | Backward Record Pointer. Record number of the last (final) summary record in the file.14 |
| 84 | FREE | int | 4 | First Free Address. The 1-based word address where the next array's data will begin.14 |
| 88 | LOCFMT | char | 8 | Location Format Word. Indicates the binary numeric format ('LTL-IEEE' or 'BIG-IEEE').14 |
| 96 | PRENUL | char | 603 | Pre-Null padding. Reserved space, typically filled with null characters.14 |
| 699 | FTPSTR | char | 28 | FTP Validation String. A specific string used to help detect corruption from improper FTP transfers.14 |
| 727 | PSTNUL | char | 297 | Post-Null padding. Additional reserved space, typically null-filled, ensuring the record totals 1024 bytes.14 |

*(Based on.14 Assumes standard C sizes: char=1 byte, int=4 bytes, double=8 bytes)*

Several fields in the File Record are fundamental to navigating and interpreting the DAF:

* **LOCIDW:** Acts as a "magic number" to confirm the file is indeed a DAF and provides a hint about its contents (e.g., SPK).14 SPICE routines check this upon opening.14  
* **ND and NI:** Define the structure of the metadata summaries associated with each data array stored in the file. These values dictate how many double-precision and integer components comprise each summary.14 Their values are constant for all summaries within a single DAF.  
* **FWARD and BWARD:** Provide the entry points into the doubly-linked list of summary records, allowing traversal from the beginning or end of the summary chain.14  
* **FREE:** Indicates the next available word address for appending new data arrays, essential for write operations.14  
* **LOCFMT:** Specifies the endianness of the binary data ('LTL-IEEE' for little-endian, 'BIG-IEEE' for big-endian), crucial for correct interpretation on different hardware architectures.14

### **2.4. Summary and Name Records**

Summary Records serve as directories for the actual data arrays (stored in Element Records). Each Summary Record contains one or more "array summaries" (also called descriptors) and control pointers to link it with other Summary Records.14

A Summary Record, like all DAF records, is 1024 bytes long and conceptually holds 128 double-precision words. The first three 8-byte words are reserved for control information 14:

1. **NEXT (Word 1):** The record number of the *next* Summary Record in the forward chain. A value of zero indicates this is the last Summary Record.  
2. **PREV (Word 2):** The record number of the *previous* Summary Record in the backward chain. A value of zero indicates this is the first Summary Record.  
3. **NSUM (Word 3):** The number of array summaries actually stored within this specific Summary Record.

It is noteworthy that although NEXT, PREV, and NSUM represent integer concepts (record numbers, counts), they are stored as double-precision values (8 bytes each) within the record.14

Following these three control words (24 bytes), the record contains the packed array summaries. Each summary describes a single data array within the DAF and consists of ND double-precision components followed by NI integer components, where ND and NI are the global values read from the File Record.14 These components are packed into a contiguous block of double-precision words within the Summary Record.14 The packing scheme is as follows:

* The first ND words of the summary store the ND double-precision components directly.  
* The subsequent words store the NI integer components. Since integers (typically 4 bytes) are smaller than doubles (8 bytes), they are packed pairwise into the 8-byte double slots. If NI is odd, the last integer component occupies the first half of the final double slot, with the second half unused.14

The total size of one packed summary, measured in 8-byte double-precision words (SS), is calculated using integer division 14:  
SS=ND+(NI+1)/2  
The number of summaries (NS) that can fit into a single Summary Record (after the 3 control words) is limited by the remaining space (125 words) 14:  
NS≤125/SS  
Summaries are stored contiguously starting from the 4th word (byte 24\) of the Summary Record and are never split across record boundaries.14  
Immediately following each Summary Record in the file is a corresponding Name Record.14 This record stores character string names associated with the arrays whose summaries are in the preceding Summary Record. The number of names in a Name Record is exactly equal to the number of summaries (NSUM) in the corresponding Summary Record.14 Each name is allocated a fixed number of characters (NC), calculated based on the summary size (SS) 14:  
NC=8×SS=8×(ND+(NI+1)/2)  
This calculation ensures that the total space occupied by the names in the Name Record is related to the space occupied by the summaries in the Summary Record. The names are stored contiguously within the 1024-byte Name Record: the first name occupies bytes 0 to NC-1, the second occupies bytes NC to 2\*NC-1, and so on.14 The tight coupling observed here, where the Name Record's structure (NC) is derived directly from the Summary Record's parameters (ND, NI via SS), and their physical adjacency in the file, suggests a design choice that simplifies file management or buffer allocation within the DAF subsystem by linking the metadata structure closely to the data structure size.

### **2.5. Element Records**

The bulk of a typical DAF file consists of Element Records.14 These records are solely dedicated to storing the actual double-precision numerical data elements that constitute the arrays described by the summaries.14 Like other DAF records, an Element Record is 1024 bytes long and can hold up to 128 double-precision (8-byte) numbers.14

Elements belonging to a single array are always stored contiguously within the DAF file's word sequence, potentially spanning multiple Element Records.14 However, a single Element Record may contain data from the end of one array and the beginning of the next if they happen to be stored sequentially.14 Element Records are typically filled completely with 128 data values, except potentially for the last Element Record containing data for a specific array if it immediately precedes a Summary Record; such a record might be only partially filled.14 The addresses (initial and final word numbers) stored within an array's summary point directly into these Element Records, specifying the location of that array's data.9

### **2.6. The Comment Area (Reserved Records)**

DAF files can optionally include a designated "comment area" located immediately after the File Record and before the first Summary Record.14 This area consists of one or more Reserved Records, allocated when the file is created.14 These records are distinct from the structured DAF components (File, Summary, Name, Element Records) and their content is generally opaque to the core DAF reading routines.14

The comment area is intended for storing human-readable, textual metadata about the file's contents, such as data sources, processing history, usage notes, or descriptive information.1 SPICE provides specific routines (e.g., dafac\_c to add comments, dafec\_c to extract comments) for managing this area.16 NAIF recommends using these dedicated routines rather than attempting to manipulate the Reserved Records directly.14 Comments are stored as lines of ASCII text, preserving internal whitespace but potentially trimming trailing blanks.25 Printable ASCII characters (decimal 32-126) are expected.25 The separation of this textual comment area from the highly structured, performance-critical numerical data sections (Summary, Name, Element Records) allows for flexible annotation without impacting the efficiency of the primary data access pathways designed for numerical arrays.

## **3\. The SPK (Spacecraft and Planet Kernel) Format**

### **3.1. Purpose and Implementation within DAF**

The SPK format is a specific application of the DAF architecture, designed explicitly for storing and retrieving ephemeris data.9 SPK stands for S(pacecraft) and P(lanet) Kernel, reflecting its purpose of handling trajectory information for a wide range of solar system objects, including spacecraft, planets, natural satellites, comets, and asteroids.1

An SPK file *is* a DAF file.1 It adheres to the fundamental DAF structure, containing a File Record, Summary Records, Name Records (though array names are often less critical for identifying SPK segments compared to the descriptor contents), Element Records, and potentially a Comment Area.1 The distinction of an SPK file is typically marked in the File Record's LOCIDW field, which usually contains the identifier DAF/SPK.14

### **3.2. Segment-Based Structure**

Within an SPK file, the ephemeris data is organized into logical units called "segments".9 Each segment encapsulates the ephemeris data for a single object (the "target") relative to another object (the "center of motion") over a continuous time interval, expressed in a specific coordinate reference frame, and represented using a particular mathematical formulation (the "SPK data type").9

A key feature of the SPK format is its ability to aggregate data for numerous objects and time spans within a single file.9 An SPK file can contain multiple segments, potentially covering different targets, centers, time ranges, reference frames, and data types. These segments do not need to be ordered chronologically within the file.9 The order in which segments appear, however, determines their priority, as discussed in Section 3.5.

### **3.3. SPK Segment Descriptors**

The metadata defining each SPK segment is stored within the DAF array summary corresponding to that segment.9 For SPK files, the DAF File Record typically specifies ND=2 and NI=6, defining the standard structure for an SPK segment descriptor.9 These values indicate that each segment's summary consists of two double-precision components and six integer components.

**Table 3: SPK Segment Descriptor Fields (ND=2, NI=6)**

| Component Type | Index (Packed) | Field Name | Description |
| :---- | :---- | :---- | :---- |
| Double | 1 | Initial Epoch | Start time of the interval covered by the segment, expressed as ephemeris seconds past the J2000 epoch (TDB).9 |
| Double | 2 | Final Epoch | End time of the interval covered by the segment, expressed as ephemeris seconds past the J2000 epoch (TDB).9 |
| Integer | 1 | Target Body ID | NAIF integer code identifying the body whose ephemeris is contained in the segment.8 |
| Integer | 2 | Center Body ID | NAIF integer code identifying the center of motion relative to which the target's state is given.9 |
| Integer | 3 | Reference Frame ID | NAIF integer code identifying the reference frame in which the state vectors are expressed.8 |
| Integer | 4 | SPK Data Type | Integer code specifying the mathematical representation of the ephemeris data (e.g., Chebyshev polynomials, Lagrange interpolation).9 |
| Integer | 5 | Initial Data Address | The 1-based DAF word address of the beginning of the ephemeris data array (in the Element Records) for this segment.9 |
| Integer | 6 | Final Data Address | The 1-based DAF word address of the end of the ephemeris data array (in the Element Records) for this segment.9 |

*(Based on.9 Indices refer to the conceptual unpacked integers.)*

The Initial and Final Data Addresses (integers 5 and 6\) are crucial as they link the metadata descriptor to the actual numerical data stored in the DAF Element Records.9 These addresses define the contiguous block of double-precision words containing the coefficients, discrete states, or other parameters specific to the segment's SPK data type.

### **3.4. SPK Data Types and Encoding**

The integer value stored as the "SPK Data Type" in the segment descriptor dictates the mathematical representation and physical storage layout of the ephemeris data within the block of Element Records pointed to by the Initial and Final Data Addresses.9 SPICE supports numerous data types, each tailored to different kinds of source data, accuracy requirements, or computational methods. The existence of these varied types reflects the evolution of orbital mechanics and data processing techniques, accommodating everything from high-precision planetary theories to satellite element sets.9 This flexibility allows the SPK format to serve as a versatile container for diverse trajectory representations.

**Table 4: Common SPK Data Types and Data Array Formats**

| Type | Name | Description & Data Array Structure (Simplified) | Reference |
| :---- | :---- | :---- | :---- |
| 2 | Chebyshev (Position only) | Position represented by Chebyshev polynomials over fixed-length time intervals. Array contains records with interval midpoint/radius and X, Y, Z coefficients, followed by a directory (initial epoch, interval length, record size, record count). | 9 |
| 3 | Chebyshev (Position & Velocity) | Similar to Type 2, but includes coefficients for X, Y, Z velocity components as well. | 9 |
| 5 | Discrete States (Two-Body) | Discrete state vectors (position/velocity) propagated using two-body mechanics. Array contains states, epochs, epoch directory, central body GM, and state count. | 9 |
| 8 | Lagrange Interpolation (Equal Steps) | Discrete states at equally spaced epochs, interpolated using Lagrange polynomials. Array contains states, first epoch, step size, polynomial degree, and state count. | 9 |
| 9 | Lagrange Interpolation (Unequal Steps) | Similar to Type 8, but epochs are unevenly spaced. Array contains states, epochs, epoch directory, polynomial degree, and state count. | 9 |
| 10 | Space Command TLE | Models Earth satellites using Two-Line Element sets. Array contains geophysical constants, TLE data packets, reference epochs, and epoch directory. | 9 |
| 12 | Hermite Interpolation (Equal Steps) | Discrete states at equally spaced epochs, interpolated using Hermite polynomials (using position and velocity). Array contains states, first epoch, step size, window size, and state count. | 9 |
| 13 | Hermite Interpolation (Unequal Steps) | Similar to Type 12, but epochs are unevenly spaced. Array contains states, epochs, epoch directory, window size, and state count. | 9 |
| 14 | Chebyshev (Unequal Steps) | Position/velocity via Chebyshev polynomials over variable time intervals using a generic segment structure. Array contains constants, data packets (midpoint, radius, coefficients), reference epochs, epoch directory, metadata. | 9 |
| 21 | Extended Modified Difference Arrays | Similar to older Type 1 (Modified Difference Arrays), but allows larger, higher-degree representations. Array contains difference line records, final epochs, epoch directory, line size, record count. | 9 |

*(Based primarily on 9, with type usage context from.26 This table is illustrative, not exhaustive; refer to SPK Required Reading for full details.)*

When software like the SPICE Toolkit or python-jplephem retrieves a state vector, it uses the SPK Data Type from the descriptor to determine which interpolation or evaluation algorithm to apply to the data read from the corresponding Element Records.9

### **3.5. Segment Priority and Data Selection**

An SPK file may contain multiple segments providing data for the same target body relative to the same center, potentially with overlapping time coverage. In such cases, a precedence rule applies: the segment located physically later in the file takes priority over segments appearing earlier.2 This mechanism allows newer or more accurate data to supersede older data simply by appending the new segment(s) to the end of the file.

When a request for ephemeris data is made (e.g., via SPICE's spkezr\_c function), the reading software searches through all loaded SPK files and their segments to find those matching the requested target, center, and time.9 If multiple matching segments are found, the reader automatically selects the one with the highest priority (i.e., the one occurring latest in the highest priority file loaded) that covers the requested epoch.9 This selection process is typically transparent to the end-user of high-level reader functions. The order of loading files also matters, with later-loaded files generally taking precedence over earlier ones for the same data.2 Meta-kernels are often used to manage the loading order and ensure correct precedence.3

## **4\. Data Content: What SPK Files Contain (and What They Don't)**

### **4.1. Included Data**

The primary and defining content of SPK files is ephemeris data: time-tagged state vectors, typically consisting of Cartesian position (x,y,z) and velocity (x˙,y˙​,z˙), for celestial bodies or spacecraft.1 This data represents the trajectory of a target object relative to a specified center of motion, expressed within a particular reference frame, and valid over a defined time interval.9

In addition to the core numerical trajectory data stored in Element Records, SPK files contain essential metadata:

* **Segment Descriptors:** Stored in DAF summaries, these provide the crucial context for each data segment: target ID, center ID, frame ID, time coverage, data type, and pointers to the data array.9  
* **File Record:** Contains global file information, including the DAF/SPK identifier, internal file name, summary format definition (ND, NI), pointers to the summary list, and binary format identifier.14  
* **Comment Area:** Optionally contains extensive human-readable metadata, such as descriptions of the data source, generation methods, intended use, accuracy assessments, contact information, and references.1 Utilities like commnt or brief can be used to view this information.15

### **4.2. Excluded Data (Requires Other Kernel Types)**

It is critical to understand that SPK files are specialized for trajectory information (position and velocity) and **do not** contain other types of ancillary data required for many space science applications. This information must be obtained from other SPICE kernel types:

* **Orientation/Attitude:** SPK files provide *where* an object is, but not *how it is oriented* in space. Time-varying orientation data (e.g., spacecraft attitude, instrument pointing) is stored in C-Kernels (CK).3  
* **Body Constants/Shape/Orientation Models:** Information about the physical properties of celestial bodies, such as their radii, flattening coefficients, gravitational parameters (GM), spin axis direction, and prime meridian definition, is contained in Planetary Constants Kernels (PCK).3 While some SPK types might embed a specific GM value (e.g., Type 5 9), comprehensive body models reside in PCKs. Binary PCKs for bodies like Earth and Moon also utilize the DAF format.9  
* **Instrument Parameters:** Details about scientific instruments, such as field-of-view geometry, focal length, mounting alignment relative to the spacecraft structure, and detector characteristics, are specified in Instrument Kernels (IK).3  
* **Reference Frame Definitions:** While SPK segment descriptors contain integer IDs for reference frames 9, the actual *definitions* of these frames (e.g., how 'J2000' relates to 'IAU\_MARS' or a spacecraft-fixed frame) are provided by Frames Kernels (FK).3 FKs establish the hierarchy and transformation rules between different coordinate systems.  
* **Time System Correlations:** SPK segment times are typically given in Ephemeris Time (ET/TDB).9 Converting between ET and other time systems like Coordinated Universal Time (UTC) requires information about leap seconds, which is stored in Leapseconds Kernels (LSK).2 Relating spacecraft onboard clock time (SCLK) to ET requires Spacecraft Clock Kernels (SCLK).3

This modular design of the SPICE kernel system is a deliberate and powerful feature.3 Different types of data often originate from different sources and have different update cycles. For instance, spacecraft trajectory (SPK) and attitude (CK) might be updated daily or weekly by navigation teams 1, while fundamental planetary constants (PCK) or leap second definitions (LSK) change much less frequently.10 Separating these data types into distinct kernels allows users to independently update and combine the latest available information for each category without needing to manage enormous, monolithic files containing all possible data. This approach enhances flexibility, simplifies data management, and improves efficiency.

### **4.3. Combining Kernels**

Because SPK files contain only a subset of the necessary information, practical applications involving observation geometry calculations (e.g., determining the apparent position of a planet as seen from a spacecraft instrument) invariably require loading multiple kernel types simultaneously.6 The SPICE Toolkit provides mechanisms for this, typically using the furnsh\_c routine (or equivalent in other languages) to load a list of required kernels.6 Often, this list is managed using a "meta-kernel" (MK) file, which is a text kernel that specifies the paths to all other kernels needed for a particular application or study.3 The meta-kernel allows users to easily manage complex sets of kernels and control their loading order, which is crucial for ensuring correct data precedence.3

While libraries like python-jplephem focus primarily on extracting data from SPK files 17, higher-level astronomy libraries built upon it, such as Skyfield 18, or alternative SPICE wrappers like SpiceyPy 30, provide frameworks that integrate data from multiple kernel types (SPK, PCK, LSK, etc.) to perform more complex calculations.

## **5\. Extracting Ephemeris Data**

### **5.1. Overview of Access Methods**

Accessing the ephemeris data stored within a binary SPK file requires navigating the DAF structure and interpreting the specific SPK data type format. While direct byte-level parsing of the file is theoretically possible given the specifications outlined in Sections 2 and 3, it is a complex and error-prone undertaking.9 Challenges include correctly handling the DAF record linking, parsing packed summary formats, managing potential binary format differences (endianness) 14, and implementing the diverse mathematical algorithms associated with each SPK data type.9

Consequently, the standard and recommended practice is to utilize established software libraries designed specifically for this purpose.3 These libraries encapsulate the low-level file parsing and computational details, providing robust and validated interfaces for accessing ephemeris data.

### **5.2. Method 1: NASA SPICE Toolkit (Conceptual)**

The official NASA SPICE Toolkit is the reference implementation for interacting with all types of SPICE kernels, including SPK files.5 The typical workflow involves:

1. **Loading Kernels:** The necessary SPK files, along with any required CK, PCK, LSK, FK, etc., are made known to the SPICE system using a loading routine like furnsh\_c (C), furnsh\_ (Fortran), or equivalents.6 This is often done by providing the path to a meta-kernel file that lists all the required kernels.3 Loading registers the files and reads essential metadata, but typically does not load the bulk ephemeris data into memory immediately.3  
2. **Requesting State Vectors:** High-level reader functions, such as spkezr\_c (state, including light time and stellar aberration corrections) or spkpos\_c (geometric position only), are called to retrieve the state vector (position and velocity) of a specified target body relative to an observing body, in a desired reference frame, at a particular ephemeris time.9 Options exist to request geometric, light-time corrected, or stellar-aberration corrected states.8  
3. **Internal Processing:** These high-level functions orchestrate the data retrieval process internally. They search through the loaded kernels to find the highest-priority segment(s) matching the request (target, center, time, frame).9 They then utilize lower-level DAF access routines (e.g., dafgsr\_c to get summary records, dafgda\_c to get data from element records \- note dafrda\_c is deprecated 21) to read the necessary metadata and ephemeris data (coefficients, states) from the file.16 Finally, they apply the appropriate interpolation or evaluation algorithm based on the segment's SPK data type to compute the state vector at the precise requested epoch.9 If the SPK file's binary format (LOCFMT) does not match the native format of the machine running the code, the SPICE library automatically handles the necessary byte swapping during the read operations.9

### **5.3. Method 2: python-jplephem**

For users working within the Python ecosystem, python-jplephem offers a popular, pure-Python alternative specifically for reading SPK files.17 It does not require the installation of the C or Fortran SPICE toolkits but depends on the NumPy library for numerical operations.17

The typical workflow with jplephem is:

1. **Loading the Kernel:** An SPK file is opened using SPK.open('filename.bsp'). This creates an SPK object, parses the DAF file structure (using the jplephem.daf module internally 17), reads the File Record, and identifies all available SPK segments, storing them as Segment objects within the kernel.segments list.17  
2. **Accessing Segments:** Users can iterate through the kernel.segments list to examine all segments or select a specific segment using dictionary-style indexing with NAIF integer IDs for the center and target, e.g., segment \= kernel\[center\_id, target\_id\].17 Metadata for a segment, such as its time coverage (start\_jd, end\_jd) or target/center IDs, can be accessed as attributes of the Segment object.26  
3. **Computing States:** Once a segment is selected, its compute(jd) method can be called to calculate the position vector, or compute\_and\_differentiate(jd) to calculate both position and velocity vectors, at a specified Julian Date (interpreted as TDB).17 These methods accept scalar Julian dates or NumPy arrays, returning corresponding NumPy arrays of state vectors.17  
4. **Efficiency:** A key feature of jplephem is its use of memory mapping (mmap on supported operating systems).17 This allows the library to access data directly from the .bsp file on disk without loading the entire, potentially very large, file into RAM. Data (e.g., Chebyshev coefficients) is read only when needed for a computation. The library leverages NumPy arrays that provide a view onto the memory-mapped file content, enabling efficient numerical processing.17  
5. **Supported Types:** jplephem primarily supports the most common SPK data types used in JPL planetary and satellite ephemerides, namely Type 2 (Chebyshev position) and Type 3 (Chebyshev position and velocity).26 Support for other types, like Type 1 or Type 21, may require separate third-party libraries.19  
6. **Utilities:** jplephem also provides command-line utilities for inspecting SPK files (python \-m jplephem comment \<file\>, ... daf \<file\>, ... spk \<file\>) and for creating smaller excerpt files containing data for a specific time range or set of targets (python \-m jplephem excerpt...).17 The excerpt utility can efficiently download only the necessary portions of a large SPK file from a URL using HTTP range requests.17

The design of python-jplephem, leveraging memory mapping and NumPy, provides a performant and Pythonic interface focused specifically on SPK data retrieval. Its clear abstraction of the DAF layer and focus on common SPK types make it suitable for tasks requiring only ephemeris data. However, for comprehensive astronomical calculations involving coordinate transformations, time conversions, or orientation data, it is typically used as a component within broader libraries like Skyfield, which integrate its capabilities with functionalities equivalent to other SPICE kernel types.18

## **6\. Pseudo RFC-Style Byte Layout Summary**

This section provides a consolidated summary of the byte-level layout of the DAF/SPK file structure, intended for developers needing a low-level understanding for parsing or implementation purposes. It assumes standard C data type sizes (char: 1 byte, int: 4 bytes, double: 8 bytes) and IEEE-754 64-bit double-precision representation, consistent with SPICE documentation.14

### **6.1. Overall File Structure**

A typical DAF/SPK file consists of a sequence of 1024-byte records organized as follows:

1. **File Record:** (1 record, Bytes 0-1023) Contains global file metadata.14  
2. **Reserved Records (Comment Area):** (Optional, N records, Bytes 1024 to 1024\*(N+1)-1) Stores textual comments.14 The number of reserved records is determined at file creation.  
3. **Data Area:** (Remaining records) Contains an interleaved sequence of Summary Records, Name Records, and Element Records holding the segment descriptors and ephemeris data.14  
   * The Summary Records form a doubly-linked list, starting at the record number specified by FWARD in the File Record and ending at the record specified by BWARD.14 Each Summary Record contains NEXT and PREV pointers (record numbers) to navigate this list.14  
   * Each Summary Record is immediately followed by its corresponding Name Record.14  
   * Element Records contain the bulk numerical data, referenced by addresses stored in the summaries.9

### **6.2. Record Layouts**

* **File Record (Bytes 0-1023):**  
  * Byte 0: LOCIDW (char) \- File identifier ('DAF/SPK')  
  * Byte 8: ND (int) \- Number of doubles per summary (e.g., 2 for SPK)  
  * Byte 12: NI (int) \- Number of integers per summary (e.g., 6 for SPK)  
  * Byte 16: LOCIFN (char) \- Internal file name  
  * Byte 76: FWARD (int) \- Record number of first summary record  
  * Byte 80: BWARD (int) \- Record number of last summary record  
  * Byte 84: FREE (int) \- First free word address (1-based)  
  * Byte 88: LOCFMT (char) \- Binary format ('LTL-IEEE' or 'BIG-IEEE')  
  * Byte 96: PRENUL (char) \- Padding  
  * Byte 699: FTPSTR (char) \- FTP validation string  
  * Byte 727: PSTNUL (char) \- Padding  
  * *(Reference: Table 114)*  
* **Summary Record (1024 bytes / 128 doubles):**  
  * Bytes 0-7 (Word 1): NEXT (double) \- Record number of next summary record (0 if last).14  
  * Bytes 8-15 (Word 2): PREV (double) \- Record number of previous summary record (0 if first).14  
  * Bytes 16-23 (Word 3): NSUM (double) \- Number of summaries in this record.14  
  * Bytes 24 onwards (Words 4-128): Packed summaries.  
    * Each summary occupies SS=ND+(NI+1)//2 double-precision words (SS \* 8 bytes).14  
    * Layout within one summary:  
      * First ND doubles: The double-precision components.  
      * Next (NI+1)//2 doubles: Packed integer components (two 4-byte integers per 8-byte double slot, potentially with padding if NI is odd).14  
* **Name Record (1024 bytes):**  
  * Contains NSUM names, corresponding to the summaries in the preceding Summary Record.14  
  * Each name occupies NC=8×SS characters.14  
  * Layout:  
    * Bytes 0 to NC-1: Name 1 (ASCII characters)  
    * Bytes NC to 2\*NC-1: Name 2 (ASCII characters)  
    * ... and so on for NSUM names.  
* **Element Record (1024 bytes / 128 doubles):**  
  * Contains up to 128 double-precision data values.14  
  * The specific structure and meaning of the data depend entirely on the SPK Data Type specified in the corresponding segment descriptor and the range defined by the Initial and Final Data Addresses stored therein.9 Data can include Chebyshev coefficients, discrete state vectors, constants, pointers, etc., arranged according to the specific type's format.9

### **6.3. Endianness and Data Representation**

* **Endianness:** The byte order for all multi-byte numerical data (doubles, integers within summaries, data in element records) is determined by the LOCFMT field in the File Record ('LTL-IEEE' for little-endian, 'BIG-IEEE' for big-endian).14 Software reading DAF files must respect this field and perform byte swapping if the file format is non-native to the reading platform.9  
* **Data Types:**  
  * Double Precision: Typically IEEE-754 64-bit floating-point numbers.14  
  * Integer: Typically 32-bit signed integers.14  
  * Character: Typically 8-bit ASCII characters.14

## **7\. Conclusion**

### **7.1. Summary of DAF/SPK Format**

The JPL SPICE Kernel (SPK) format, commonly found in .bsp files, is a highly structured binary format built upon the Double precision Array File (DAF) architecture. DAF provides a portable and efficient mechanism for storing and accessing large arrays of double-precision numbers using a system of fixed-size records categorized as File, Summary, Name, Element, and optional Reserved (Comment) records.14 SPK files leverage this DAF structure to store ephemeris data (position and velocity) organized into segments.9 Each segment is described by a DAF summary (descriptor) containing metadata such as target/center/frame IDs, time coverage, and pointers to the actual numerical data (e.g., polynomial coefficients, discrete states) stored in Element Records.9 The specific encoding and interpretation of this numerical data are determined by an SPK Data Type code within the descriptor, allowing the format to accommodate various mathematical representations of trajectories.9

### **7.2. Key Takeaways**

Several key points emerge from this technical examination:

* **Content Specialization:** SPK files are specifically designed for position and velocity data. Comprehensive geometric analysis requires integrating SPK data with information from other SPICE kernel types like CK (orientation), PCK (body constants/shape), FK (frame definitions), and LSK (time).3  
* **Metadata Dependency:** Correct interpretation of SPK data relies heavily on the metadata embedded within the file, particularly the File Record (for global parameters like ND/NI and binary format) and the segment descriptors (for target/center/frame IDs, time span, data type, and data location).9 The optional comment area can provide crucial contextual information.15  
* **Tooling Necessity:** Due to the binary nature, the specific DAF record structure, packed summary formats, potential endianness issues, and the variety of SPK data types, direct manual parsing is impractical and error-prone. Reliable data extraction necessitates the use of specialized software like the NASA SPICE Toolkit or dedicated libraries such as python-jplephem.3  
* **Modularity Benefit:** The separation of different data types (ephemeris, orientation, constants, time, etc.) into distinct kernel files (SPK, CK, PCK, LSK, etc.) is a fundamental aspect of the SPICE system's design. This modularity allows for independent updates, flexible data combination, and efficient management of diverse ancillary datasets common in space science.3

### **7.3. Final Remarks**

The DAF/SPK format represents a robust and versatile solution for storing and accessing large volumes of precise ephemeris data, refined over decades of use in space exploration missions. While its internal structure is complex, understanding the DAF architecture and the SPK segment/descriptor model provides essential insight into how trajectory information is organized and accessed. For definitive specifications and details on specific SPK data types or SPICE Toolkit functionalities, consulting the official NAIF Required Reading documents (such as daf.req and spk.req) remains paramount.1 Tools like python-jplephem provide valuable, practical means for interacting with these files within modern data analysis environments.17

#### **Works cited**

1. pds-geosciences.wustl.edu, accessed May 4, 2025, [https://pds-geosciences.wustl.edu/grail/grail-l-rss-2-edr-v1/grail\_0201/document/spk\_mm\_sis.htm](https://pds-geosciences.wustl.edu/grail/grail-l-rss-2-edr-v1/grail_0201/document/spk_mm_sis.htm)  
2. SPICE Files \- a.i. solutions, accessed May 4, 2025, [https://ai-solutions.com/\_help\_Files/spice\_files.htm](https://ai-solutions.com/_help_Files/spice_files.htm)  
3. Introduction to Kernels \- FTP Directory Listing \- NASA, accessed May 4, 2025, [https://naif.jpl.nasa.gov/pub/naif/toolkit\_docs/Tutorials/pdf/individual\_docs/12\_intro\_to\_kernels.pdf](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/Tutorials/pdf/individual_docs/12_intro_to_kernels.pdf)  
4. SPICE kernels — PlanetMapper 1.12.4 documentation \- Read the Docs, accessed May 4, 2025, [https://planetmapper.readthedocs.io/en/stable/spice\_kernels.html](https://planetmapper.readthedocs.io/en/stable/spice_kernels.html)  
5. The Lucy SPICE Data Archive \- FTP Directory Listing \- NASA, accessed May 4, 2025, [https://naif.jpl.nasa.gov/pub/naif/pds/pds4/lucy/lucy\_spice/document/spiceds\_v001.html](https://naif.jpl.nasa.gov/pub/naif/pds/pds4/lucy/lucy_spice/document/spiceds_v001.html)  
6. Introduction to SPICE \- NASA, accessed May 4, 2025, [https://naif.jpl.nasa.gov/pub/naif/toolkit\_docs/C/info/intrdctn.html](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/info/intrdctn.html)  
7. SPICE Kernel Required Reading \- FTP Directory Listing, accessed May 4, 2025, [https://naif.jpl.nasa.gov/pub/naif/toolkit\_docs/C/req/kernel.html](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/kernel.html)  
8. Required Reading, accessed May 4, 2025, [https://naif.jpl.nasa.gov/pub/naif/toolkit\_docs/C/req/index.html](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/index.html)  
9. SPK Required Reading \- FTP Directory Listing, accessed May 4, 2025, [https://naif.jpl.nasa.gov/pub/naif/toolkit\_docs/C/req/spk.html](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/spk.html)  
10. SPK Required Reading \- FTP Directory Listing, accessed May 4, 2025, [https://naif.jpl.nasa.gov/pub/naif/toolkit\_docs/FORTRAN/req/spk.html](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/FORTRAN/req/spk.html)  
11. How to convert a SPICE SPK kernel into human-readable data using SPICE toolkit and utilities \- Space Exploration Stack Exchange, accessed May 4, 2025, [https://space.stackexchange.com/questions/48105/how-to-convert-a-spice-spk-kernel-into-human-readable-data-using-spice-toolkit-a](https://space.stackexchange.com/questions/48105/how-to-convert-a-spice-spk-kernel-into-human-readable-data-using-spice-toolkit-a)  
12. Generic Kernels \- NASA, accessed May 4, 2025, [https://naif.jpl.nasa.gov/naif/data\_generic.html](https://naif.jpl.nasa.gov/naif/data_generic.html)  
13. SPICE Documentation Taxonomy, accessed May 4, 2025, [https://spiftp.esac.esa.int/workshops/2006\_06\_ESTEC\_TUTORAL/SPICE\_Tutorials\_PDF/31\_docs\_taxonomy.pdf](https://spiftp.esac.esa.int/workshops/2006_06_ESTEC_TUTORAL/SPICE_Tutorials_PDF/31_docs_taxonomy.pdf)  
14. Double Precision Array Files (DAF) \- FTP Directory Listing \- NASA, accessed May 4, 2025, [https://naif.jpl.nasa.gov/pub/naif/toolkit\_docs/C/req/daf.html](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/daf.html)  
15. Questions About a Specific SPICE Kernel \- NASA, accessed May 4, 2025, [https://naif.jpl.nasa.gov/naif/specificspicekernel.html](https://naif.jpl.nasa.gov/naif/specificspicekernel.html)  
16. CSPICE functions, accessed May 4, 2025, [https://naif.jpl.nasa.gov/pub/naif/toolkit\_docs/C/cspice/](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/)  
17. jplephem \- PyPI, accessed May 4, 2025, [https://pypi.org/project/jplephem/](https://pypi.org/project/jplephem/)  
18. README.md \- brandon-rhodes/python-jplephem \- GitHub, accessed May 4, 2025, [https://github.com/brandon-rhodes/python-jplephem/blob/master/README.md](https://github.com/brandon-rhodes/python-jplephem/blob/master/README.md)  
19. JPL DE Documentation \- Astronomy Stack Exchange, accessed May 4, 2025, [https://astronomy.stackexchange.com/questions/20377/jpl-de-documentation](https://astronomy.stackexchange.com/questions/20377/jpl-de-documentation)  
20. brandon-rhodes/python-jplephem: Python version of NASA DE4xx ephemerides, the basis for the Astronomical Alamanac \- GitHub, accessed May 4, 2025, [https://github.com/brandon-rhodes/python-jplephem](https://github.com/brandon-rhodes/python-jplephem)  
21. dafrda\_c, accessed May 4, 2025, [https://naif.jpl.nasa.gov/pub/naif/toolkit\_docs/C/cspice/dafrda\_c.html](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/dafrda_c.html)  
22. dafopw\_c, accessed May 4, 2025, [https://naif.jpl.nasa.gov/pub/naif/toolkit\_docs/C/cspice/dafopw\_c.html](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/dafopw_c.html)  
23. dafopr\_c, accessed May 4, 2025, [https://naif.jpl.nasa.gov/pub/naif/toolkit\_docs/C/cspice/dafopr\_c.html](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/dafopr_c.html)  
24. How to read DAF (double precision array file) "transfer" files? \- Stack Overflow, accessed May 4, 2025, [https://stackoverflow.com/questions/21511224/how-to-read-daf-double-precision-array-file-transfer-files](https://stackoverflow.com/questions/21511224/how-to-read-daf-double-precision-array-file-transfer-files)  
25. dafac\_c, accessed May 4, 2025, [https://naif.jpl.nasa.gov/pub/naif/toolkit\_docs/C/cspice/dafac\_c.html](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/dafac_c.html)  
26. python-jplephem/jplephem/spk.py at master · brandon-rhodes/python-jplephem \- GitHub, accessed May 4, 2025, [https://github.com/brandon-rhodes/python-jplephem/blob/master/jplephem/spk.py](https://github.com/brandon-rhodes/python-jplephem/blob/master/jplephem/spk.py)  
27. python-jplephem/jplephem/\_\_init\_\_.py at master · brandon-rhodes/python-jplephem \- GitHub, accessed May 4, 2025, [https://github.com/brandon-rhodes/python-jplephem/blob/master/jplephem/\_\_init\_\_.py](https://github.com/brandon-rhodes/python-jplephem/blob/master/jplephem/__init__.py)  
28. Planets and their moons: JPL ephemeris files — Skyfield documentation \- Rhodes Mill, accessed May 4, 2025, [https://rhodesmill.org/skyfield/planets.html](https://rhodesmill.org/skyfield/planets.html)  
29. Where are jplephem ephemerides api documented? \- Stack Overflow, accessed May 4, 2025, [https://stackoverflow.com/questions/25058943/where-are-jplephem-ephemerides-api-documented](https://stackoverflow.com/questions/25058943/where-are-jplephem-ephemerides-api-documented)  
30. JPL Horizons on-line solar system data and ephemeris computation service | Hacker News, accessed May 4, 2025, [https://news.ycombinator.com/item?id=42549195](https://news.ycombinator.com/item?id=42549195)  
31. Extract date range of a bsp · Issue \#443 · skyfielders/python-skyfield \- GitHub, accessed May 4, 2025, [https://github.com/skyfielders/python-skyfield/issues/443](https://github.com/skyfielders/python-skyfield/issues/443)  
32. dafgsr\_c, accessed May 4, 2025, [https://naif.jpl.nasa.gov/pub/naif/toolkit\_docs/C/cspice/dafgsr\_c.html](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/dafgsr_c.html)  
33. jplephem \- PyPI, accessed May 4, 2025, [https://pypi.org/project/jplephem/1.2/](https://pypi.org/project/jplephem/1.2/)  
34. python-jplephem/jplephem/daf.py at master · brandon-rhodes/python-jplephem \- GitHub, accessed May 4, 2025, [https://github.com/brandon-rhodes/python-jplephem/blob/master/jplephem/daf.py](https://github.com/brandon-rhodes/python-jplephem/blob/master/jplephem/daf.py)  
35. Error when opening de405.bsp · Issue \#12 · brandon-rhodes/python-jplephem \- GitHub, accessed May 4, 2025, [https://github.com/brandon-rhodes/python-jplephem/issues/12](https://github.com/brandon-rhodes/python-jplephem/issues/12)  
36. error "no module named jplephem.pck" when trying to use skyfield \- Stack Overflow, accessed May 4, 2025, [https://stackoverflow.com/questions/59563016/error-no-module-named-jplephem-pck-when-trying-to-use-skyfield](https://stackoverflow.com/questions/59563016/error-no-module-named-jplephem-pck-when-trying-to-use-skyfield)  
37. Installing Skyfield \- Rhodes Mill, accessed May 4, 2025, [https://rhodesmill.org/skyfield/installation.html](https://rhodesmill.org/skyfield/installation.html)
//! Standard SPICE target names and ID numbers
//!
//! This module provides mappings between celestial body names and ID numbers
//! used in the JPL ephemerides.

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    /// Map from target ID numbers to canonical names
    static ref TARGET_NAMES: HashMap<i32, &'static str> = {
        let mut m = HashMap::new();
        for &(id, name) in TARGET_NAME_PAIRS.iter() {
            m.insert(id, name);
        }
        m
    };

    /// Map from lowercase target names to ID numbers
    static ref TARGET_IDS: HashMap<String, i32> = {
        let mut m = HashMap::new();
        for &(id, name) in TARGET_NAME_PAIRS.iter() {
            m.insert(name.to_lowercase(), id);
        }
        m
    };
}

/// Get the name of a target given its ID number
pub fn target_name(id: i32) -> Option<&'static str> {
    TARGET_NAMES.get(&id).copied()
}

/// Get the ID number of a target given its name
pub fn target_id(name: &str) -> Option<i32> {
    TARGET_IDS.get(&name.to_lowercase()).copied()
}

/// Title-case a target name if it looks safe to do so
pub fn titlecase(name: &str) -> String {
    if name.starts_with(['1', 'C', 'D']) {
        name.to_string()
    } else {
        name.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c
                        .to_uppercase()
                        .chain(chars.flat_map(|c| c.to_lowercase()))
                        .collect(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    }
}

/// Pairs of (id, name) for celestial bodies
const TARGET_NAME_PAIRS: &[(i32, &str)] = &[
    (0, "SOLAR_SYSTEM_BARYCENTER"),
    (0, "SSB"),
    (0, "SOLAR SYSTEM BARYCENTER"),
    (1, "MERCURY_BARYCENTER"),
    (1, "MERCURY BARYCENTER"),
    (2, "VENUS_BARYCENTER"),
    (2, "VENUS BARYCENTER"),
    (3, "EARTH_BARYCENTER"),
    (3, "EMB"),
    (3, "EARTH MOON BARYCENTER"),
    (3, "EARTH-MOON BARYCENTER"),
    (3, "EARTH BARYCENTER"),
    (4, "MARS_BARYCENTER"),
    (4, "MARS BARYCENTER"),
    (5, "JUPITER_BARYCENTER"),
    (5, "JUPITER BARYCENTER"),
    (6, "SATURN_BARYCENTER"),
    (6, "SATURN BARYCENTER"),
    (7, "URANUS_BARYCENTER"),
    (7, "URANUS BARYCENTER"),
    (8, "NEPTUNE_BARYCENTER"),
    (8, "NEPTUNE BARYCENTER"),
    (9, "PLUTO_BARYCENTER"),
    (9, "PLUTO BARYCENTER"),
    (10, "SUN"),
    (199, "MERCURY"),
    (299, "VENUS"),
    (399, "EARTH"),
    (301, "MOON"),
    (499, "MARS"),
    (401, "PHOBOS"),
    (402, "DEIMOS"),
    (599, "JUPITER"),
    (501, "IO"),
    (502, "EUROPA"),
    (503, "GANYMEDE"),
    (504, "CALLISTO"),
    // Additional entries would go here, ported from the Python version
];

/// Common target name/ID pairs used in applications
pub mod targets {

    /// Solar System Barycenter
    pub const SOLAR_SYSTEM_BARYCENTER: i32 = 0;
    /// Mercury Barycenter
    pub const MERCURY_BARYCENTER: i32 = 1;
    /// Venus Barycenter
    pub const VENUS_BARYCENTER: i32 = 2;
    /// Earth-Moon Barycenter
    pub const EARTH_MOON_BARYCENTER: i32 = 3;
    /// Mars Barycenter
    pub const MARS_BARYCENTER: i32 = 4;
    /// Jupiter Barycenter
    pub const JUPITER_BARYCENTER: i32 = 5;
    /// Saturn Barycenter
    pub const SATURN_BARYCENTER: i32 = 6;
    /// Uranus Barycenter
    pub const URANUS_BARYCENTER: i32 = 7;
    /// Neptune Barycenter
    pub const NEPTUNE_BARYCENTER: i32 = 8;
    /// Pluto Barycenter
    pub const PLUTO_BARYCENTER: i32 = 9;
    /// Sun
    pub const SUN: i32 = 10;
    /// Mercury
    pub const MERCURY: i32 = 199;
    /// Venus
    pub const VENUS: i32 = 299;
    /// Earth
    pub const EARTH: i32 = 399;
    /// Moon
    pub const MOON: i32 = 301;
    /// Mars
    pub const MARS: i32 = 499;
}

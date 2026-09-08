//! Python comparison tests for the text kernel parser.
//!
//! These compare [`PlanetaryConstants::read_text`] against Skyfield's
//! `skyfield.planetarylib.PlanetaryConstants.read_text` on the checked-in
//! excerpt of `pck00011.tpc`, so both parsers see exactly the same bytes.

#[cfg(test)]
mod tests {
    use crate::planetarylib::{KernelValue, PlanetaryConstants};
    use crate::pybridge::{PyRustBridge, PythonResult};
    use std::collections::HashMap;

    /// The excerpt both parsers read, relative to the crate root.
    const EXCERPT_PATH: &str = "src/planetarylib/pck00011_excerpt.tpc";

    fn unwrap_py_json(raw: &str) -> serde_json::Value {
        let inner = match PythonResult::try_from(raw).expect("Failed to parse Python result") {
            PythonResult::String(s) => s,
            other => panic!("Expected String result, got {:?}", other),
        };
        serde_json::from_str(&inner).expect("JSON parse failed")
    }

    /// Every variable Skyfield reads out of the excerpt, as a flat list of
    /// numbers per name.
    fn skyfield_variables() -> HashMap<String, Vec<f64>> {
        let bridge = PyRustBridge::new().expect("Failed to create Python bridge");
        let code = format!(
            r#"
import json
from skyfield.planetarylib import PlanetaryConstants

pc = PlanetaryConstants()
with open('{path}', 'rb') as f:
    pc.read_text(f)

out = {{}}
for name, value in pc.variables.items():
    if not isinstance(value, list):
        value = [value]
    out[name] = [float(x) for x in value]

rust.collect_string(json.dumps(out))
"#,
            path = EXCERPT_PATH,
        );

        let py_result = bridge
            .run_py_to_json(&code)
            .expect("Skyfield text kernel parse failed");

        let parsed = unwrap_py_json(&py_result);
        let object = parsed.as_object().expect("expected a JSON object");
        object
            .iter()
            .map(|(name, value)| {
                let numbers = value
                    .as_array()
                    .expect("expected a JSON array")
                    .iter()
                    .map(|x| x.as_f64().expect("expected a JSON number"))
                    .collect();
                (name.clone(), numbers)
            })
            .collect()
    }

    /// The parser reproduces Skyfield's variables dict name for name.
    #[test]
    fn test_variables_match_skyfield() {
        let expected = skyfield_variables();
        assert!(!expected.is_empty(), "Skyfield read no variables");

        let mut pc = PlanetaryConstants::new();
        pc.open_text(EXCERPT_PATH).unwrap();

        let mut names: Vec<&String> = pc.variables.keys().collect();
        names.sort();
        let mut expected_names: Vec<&String> = expected.keys().collect();
        expected_names.sort();
        assert_eq!(names, expected_names, "variable names differ");

        for (name, want) in &expected {
            let got = pc.variables[name]
                .to_numbers()
                .unwrap_or_else(|| panic!("{} did not parse as numbers", name));
            assert_eq!(&got, want, "{} differs from Skyfield", name);
        }
    }

    /// Skyfield unboxes a lone value to a scalar, and so do we.
    #[test]
    fn test_scalars_are_unboxed_like_skyfield() {
        let mut pc = PlanetaryConstants::new();
        pc.open_text(EXCERPT_PATH).unwrap();

        assert!(matches!(
            pc.variables["BODY4_MAX_PHASE_DEGREE"],
            KernelValue::Number(_)
        ));
        assert!(matches!(
            pc.variables["BODY499_RADII"],
            KernelValue::Numbers(_)
        ));

        let bridge = PyRustBridge::new().expect("Failed to create Python bridge");
        let code = format!(
            r#"
import json
from skyfield.planetarylib import PlanetaryConstants

pc = PlanetaryConstants()
with open('{path}', 'rb') as f:
    pc.read_text(f)

rust.collect_string(json.dumps({{
    "scalar": isinstance(pc.variables['BODY4_MAX_PHASE_DEGREE'], list),
    "vector": isinstance(pc.variables['BODY499_RADII'], list),
}}))
"#,
            path = EXCERPT_PATH,
        );
        let parsed = unwrap_py_json(&bridge.run_py_to_json(&code).expect("Skyfield parse failed"));
        assert_eq!(parsed["scalar"].as_bool(), Some(false));
        assert_eq!(parsed["vector"].as_bool(), Some(true));
    }
}

//! Python comparison tests for the text kernel parser and the binary PCK
//! frames.
//!
//! These compare [`PlanetaryConstants::read_text`] against Skyfield's
//! `skyfield.planetarylib.PlanetaryConstants.read_text` on the checked-in
//! excerpt of `pck00011.tpc`, so both parsers see exactly the same bytes, and
//! compare [`PckFrame`](crate::planetarylib::PckFrame) against Skyfield's
//! `planetarylib.Frame` on the lunar principal-axes kernel. The kernel tests
//! are `#[ignore]`d because they need a download; the same numbers are checked
//! in as golden matrices in `pck_frame.rs`.

#[cfg(test)]
mod tests {
    use crate::framelib::Frame;
    use crate::planetarylib::{KernelValue, PlanetaryConstants};
    use crate::pybridge::{PyRustBridge, PythonResult};
    use std::collections::HashMap;

    /// The three epochs both implementations are asked about, as TDB Julian
    /// dates: 2000-01-01, 2010-06-15 and 2025-03-01.
    const EPOCHS: [f64; 3] = [2451544.5, 2455362.5, 2460735.5];

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

    /// `PckFrame` reproduces Skyfield's `MOON_PA_DE421` rotation matrices.
    ///
    /// Both sides read the same two kernels from the data directory, so the
    /// only thing under test is the interpolation and the assembly of the
    /// matrix.
    #[test]
    #[ignore = "downloads moon_pa_de421_1900-2050.bpc and moon_080317.tf"]
    fn test_moon_pa_frame_matches_skyfield() {
        let loader = crate::Loader::new();
        let text_path = loader.ensure_file("moon_080317.tf").unwrap();
        let binary_path = loader.ensure_file("moon_pa_de421_1900-2050.bpc").unwrap();

        let mut pc = PlanetaryConstants::new();
        pc.open_text(&text_path).unwrap();
        pc.open_binary(&binary_path).unwrap();
        let frame = pc.build_frame_named("MOON_PA_DE421").unwrap();

        let bridge = PyRustBridge::new().expect("Failed to create Python bridge");
        let code = format!(
            r#"
import json
from skyfield.api import load
from skyfield.planetarylib import PlanetaryConstants

ts = load.timescale()
pc = PlanetaryConstants()
pc.read_text(open('{text}', 'rb'))
pc.read_binary(open('{binary}', 'rb'))
frame = pc.build_frame_named('MOON_PA_DE421')

out = []
for jd in {epochs:?}:
    out.append([float(x) for x in frame.rotation_at(ts.tdb_jd(jd)).flatten()])

rust.collect_string(json.dumps(out))
"#,
            text = text_path.display(),
            binary = binary_path.display(),
            epochs = EPOCHS,
        );

        let parsed = unwrap_py_json(&bridge.run_py_to_json(&code).expect("Skyfield frame failed"));
        let rows = parsed.as_array().expect("expected a JSON array");
        assert_eq!(rows.len(), EPOCHS.len());

        let ts = crate::time::Timescale::default();
        for (jd, expected) in EPOCHS.iter().zip(rows) {
            let rotation = frame.rotation_at(&ts.tdb_jd(*jd));
            let expected = expected.as_array().expect("expected a JSON array");
            for (i, want) in expected.iter().enumerate() {
                let want = want.as_f64().expect("expected a number");
                let got = rotation[(i / 3, i % 3)];
                assert!(
                    (got - want).abs() < 1e-9,
                    "JD {jd}: element {i} is {got}, Skyfield says {want}"
                );
            }
        }
    }
}

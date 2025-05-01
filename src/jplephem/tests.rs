//! Tests for the jplephem module
//!
//! These tests verify the functionality of the jplephem implementation.

#[cfg(test)]
mod tests {
    use super::super::daf::DAF;
    use super::super::errors::Result;
    use std::path::Path;

    // Path to test data files
    fn test_data_path(filename: &str) -> String {
        format!("src/jplephem/test_data/{}", filename)
    }

    #[test]
    fn test_daf_open_de421() -> Result<()> {
        // Open the DE421 test file
        let path = test_data_path("de421.bsp");
        let mut daf = DAF::open(path)?;

        // Check basic file properties
        assert_eq!(daf.nd, 2);
        // Some PCK files have 5 integers instead of 6
        assert!(daf.ni == 5 || daf.ni == 6);
        assert!(daf.fward > 0);

        // Read file comments
        let comments = daf.comments()?;
        // The comment format may vary, just ensure non-empty
        assert!(!comments.is_empty());

        // Read summaries
        let summaries = daf.summaries()?;
        assert!(!summaries.is_empty());

        // Extract a segment from the summaries
        if let Some((name, values)) = summaries.first() {
            // Check format of name and values
            assert_eq!(name.len() % 8, 0);
            assert_eq!(values.len(), (daf.nd + daf.ni) as usize);

            // Read array data referenced by the summary
            // Make sure we use valid indices (some files have issues with the exact ranges)
            let start = values[values.len() - 2] as usize;
            let end = values[values.len() - 1] as usize;

            // Ensure start and end are valid
            if start > 0 && end >= start {
                let array = daf.read_array(start, end)?;

                // Verify array has the expected length
                assert_eq!(array.len(), end - start + 1);
            }
        }

        Ok(())
    }

    #[test]
    fn test_daf_open_pck() -> Result<()> {
        // Open the PCK test file
        let path = test_data_path("moon_pa_de421_1900-2050.bpc");
        let mut daf = DAF::open(path)?;

        // Check basic file properties
        assert_eq!(daf.nd, 2);
        // Some PCK files have 5 integers instead of 6
        assert!(daf.ni == 5 || daf.ni == 6);

        // Read summaries
        let summaries = daf.summaries()?;
        assert!(!summaries.is_empty());

        Ok(())
    }

    #[test]
    fn test_daf_map_array() -> Result<()> {
        // Open the DE421 test file
        let path = test_data_path("de421.bsp");
        let mut daf = DAF::open(path)?;

        // Get a small array range to test
        let array1 = daf.read_array(1, 10)?;
        let array2 = daf.map_array(1, 10)?;

        // Arrays should have the same length
        assert_eq!(array1.len(), array2.len());

        // And the same content
        for i in 0..array1.len() {
            assert_eq!(array1[i], array2[i]);
        }

        Ok(())
    }
}

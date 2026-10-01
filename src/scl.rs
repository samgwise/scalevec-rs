// Scala .scl file format parser and exporter.
// Spec: https://www.huygens-fokker.org/scala/scl_format.html

use super::ScaleVec;

/// Parse a Scala .scl file into a ScaleVec.
///
/// The .scl format:
/// - Lines starting with `!` are comments
/// - First non-comment line: description
/// - Second non-comment line: number of notes
/// - Remaining lines: pitch values (cents if contains `.`, else ratio)
/// - The implicit 1/1 (0 cents) root is not in the file
pub fn from_scl(text: &str) -> Result<ScaleVec, String> {
    let mut lines = text
        .lines()
        .filter(|line| !line.trim_start().starts_with('!'));

    // First non-comment line: description (we skip it for now)
    let _description = lines.next().ok_or("Missing description line")?;

    // Second non-comment line: number of notes
    let count_line = lines.next().ok_or("Missing note count line")?;
    let note_count: usize = count_line
        .trim()
        .parse()
        .map_err(|e| format!("Invalid note count '{}': {}", count_line.trim(), e))?;

    // Parse pitch values
    let mut cents_values = Vec::with_capacity(note_count + 1);
    cents_values.push(0.0); // Implicit root 1/1

    for line in lines.take(note_count) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Take only the first token (ignore trailing comments/labels)
        let token = trimmed.split_whitespace().next().unwrap_or("");

        let cents = parse_pitch_value(token)?;
        cents_values.push(cents);
    }

    if cents_values.len() < 2 {
        return Err("Scale must have at least one interval".to_string());
    }

    Ok(ScaleVec::new(cents_values, None))
}

/// Export a ScaleVec as a Scala .scl format string.
pub fn to_scl(scale: &ScaleVec, name: &str, description: &str) -> String {
    let mut output = String::new();

    // Header comment with filename
    output.push_str(&format!("! {}.scl\n", name));
    output.push_str("!\n");

    // Description
    output.push_str(description);
    output.push('\n');

    // Number of notes (excluding the implicit 1/1 root)
    let note_count = scale.vector().len() - 1;
    output.push_str(&format!(" {}\n", note_count));
    output.push_str("!\n");

    // Pitch values (skip the root, which is implicit 0 cents)
    for &value in &scale.vector()[1..] {
        output.push_str(&format!(" {:.6}\n", value));
    }

    output
}

/// Parse a single pitch value from .scl format.
///
/// - If it contains a `.`, it's a cents value
/// - If it contains a `/`, it's a ratio (converted to cents)
/// - If it contains a `\`, it's n\m EDO notation (n steps of m-EDO)
/// - If it's a plain integer, it's treated as a ratio n/1
fn parse_pitch_value(token: &str) -> Result<f64, String> {
    parse_interval(token)
}

/// Parse a single interval value from user input.
///
/// Supports all common microtonal notations:
/// - Cents: `701.955` (any value containing a `.`)
/// - Ratios: `3/2` (converted to cents via `1200 * log2(ratio)`)
/// - n\\m EDO: `7\\12` (n steps of m equal divisions = `n * 1200 / m` cents)
/// - Plain integers: `2` (treated as ratio 2/1)
pub fn parse_interval(token: &str) -> Result<f64, String> {
    let token = token.trim();
    if token.is_empty() {
        return Err("Empty interval".to_string());
    }

    if token.contains('\\') {
        // n\m EDO notation: n steps of m-EDO
        let parts: Vec<&str> = token.split('\\').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid EDO notation '{}'", token));
        }
        let n: f64 = parts[0]
            .trim()
            .parse()
            .map_err(|e| format!("Invalid step count '{}': {}", parts[0].trim(), e))?;
        let m: f64 = parts[1]
            .trim()
            .parse()
            .map_err(|e| format!("Invalid EDO divisions '{}': {}", parts[1].trim(), e))?;
        if m == 0.0 {
            return Err("Division by zero in EDO notation".to_string());
        }
        Ok(n * 1200.0 / m)
    } else if token.contains('.') {
        // Cents value
        token
            .parse::<f64>()
            .map_err(|e| format!("Invalid cents value '{}': {}", token, e))
    } else if token.contains('/') {
        // Ratio — convert to cents
        let parts: Vec<&str> = token.split('/').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid ratio '{}'", token));
        }
        let num: f64 = parts[0]
            .trim()
            .parse()
            .map_err(|e| format!("Invalid numerator '{}': {}", parts[0].trim(), e))?;
        let den: f64 = parts[1]
            .trim()
            .parse()
            .map_err(|e| format!("Invalid denominator '{}': {}", parts[1].trim(), e))?;
        if den == 0.0 {
            return Err("Division by zero in ratio".to_string());
        }
        if num / den <= 0.0 {
            return Err(format!("Negative ratio '{}'", token));
        }
        Ok(1200.0 * (num / den).log2())
    } else {
        // Plain integer — treat as ratio n/1
        let num: f64 = token
            .parse()
            .map_err(|e| format!("Invalid integer ratio '{}': {}", token, e))?;
        if num <= 0.0 {
            return Err(format!("Non-positive integer ratio '{}'", token));
        }
        Ok(1200.0 * num.log2())
    }
}

/// Parse a multi-line custom scale definition.
///
/// Each line is one interval (cents, ratio, or n\\m EDO).
/// The implicit 1/1 root is not included in the input.
/// The final line should be the period (e.g. `2/1` or `1200.`).
pub fn parse_custom_scale(text: &str) -> Result<ScaleVec, String> {
    let mut cents_values = vec![0.0]; // Implicit root

    for (line_num, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('!') {
            continue;
        }
        match parse_interval(trimmed) {
            Ok(cents) => cents_values.push(cents),
            Err(e) => return Err(format!("Line {}: {}", line_num + 1, e)),
        }
    }

    if cents_values.len() < 2 {
        return Err("Scale must have at least one interval".to_string());
    }

    Ok(ScaleVec::new(cents_values, None))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Scale;

    const EPSILON: f64 = 0.01;

    #[test]
    fn test_parse_edo_notation() {
        // 7\12 = 7 steps of 12-EDO = 700 cents (tempered fifth)
        let cents = parse_interval("7\\12").unwrap();
        assert!((cents - 700.0).abs() < EPSILON, "7\\12: got {}", cents);

        // 12\12 = 1200 cents (octave)
        let cents = parse_interval("12\\12").unwrap();
        assert!((cents - 1200.0).abs() < EPSILON);

        // 11\19 in 19-EDO
        let cents = parse_interval("11\\19").unwrap();
        let expected = 11.0 * 1200.0 / 19.0;
        assert!((cents - expected).abs() < EPSILON);
    }

    #[test]
    fn test_parse_custom_scale() {
        let text = "9/8\n5/4\n4/3\n3/2\n5/3\n15/8\n2/1";
        let scale = parse_custom_scale(text).unwrap();
        assert_eq!(scale.vector().len(), 8); // 7 intervals + root
        // 3/2 should be at index 4
        assert!((scale.vector()[4] - 701.955).abs() < EPSILON);
    }

    #[test]
    fn test_parse_custom_scale_mixed_notation() {
        // Mix of cents, ratios, and EDO
        let text = "200.\n5/4\n7\\12\n2/1";
        let scale = parse_custom_scale(text).unwrap();
        assert_eq!(scale.vector().len(), 5);
        assert!((scale.vector()[1] - 200.0).abs() < EPSILON);
        assert!((scale.vector()[3] - 700.0).abs() < EPSILON);
    }

    #[test]
    fn test_parse_cents() {
        let cents = parse_pitch_value("701.955").unwrap();
        assert!((cents - 701.955).abs() < EPSILON);
    }

    #[test]
    fn test_parse_ratio() {
        // 3/2 = 701.955 cents (perfect fifth)
        let cents = parse_pitch_value("3/2").unwrap();
        assert!((cents - 701.955).abs() < EPSILON);
    }

    #[test]
    fn test_parse_integer_ratio() {
        // 2 = 2/1 = 1200 cents (octave)
        let cents = parse_pitch_value("2").unwrap();
        assert!((cents - 1200.0).abs() < EPSILON);
    }

    #[test]
    fn test_parse_scl_meantone() {
        let scl = "\
! meanquar.scl
!
1/4-comma meantone scale. Pietro Aaron's temperament (1523)
 12
!
 76.04900
 193.15686
 310.26471
 5/4
 503.42157
 579.47057
 696.57843
 25/16
 889.73529
 1006.84314
 1082.89214
 2/1
";
        let scale = from_scl(scl).unwrap();
        assert_eq!(scale.vector().len(), 13); // 12 notes + root
        assert!((scale.vector()[0] - 0.0).abs() < EPSILON); // root

        // 5/4 should be ≈ 386.314 cents
        assert!((scale.vector()[4] - 386.314).abs() < 0.01, "5/4: got {}", scale.vector()[4]);

        // Last value (2/1) should be 1200 cents
        assert!((scale.vector()[12] - 1200.0).abs() < EPSILON);
    }

    #[test]
    fn test_parse_just_major() {
        let scl = "\
! just_major.scl
!
Just major scale
 7
!
 9/8
 5/4
 4/3
 3/2
 5/3
 15/8
 2/1
";
        let scale = from_scl(scl).unwrap();
        assert_eq!(scale.vector().len(), 8); // 7 notes + root

        // 3/2 should be ≈ 701.955 cents
        let fifth = scale.vector()[4];
        assert!((fifth - 701.955).abs() < EPSILON, "3/2: got {}", fifth);

        // Degree 0 should give 0 (root)
        assert!((scale.step(0.0) - 0.0).abs() < EPSILON);
    }

    #[test]
    fn test_scl_round_trip() {
        let original = "\
! test.scl
!
Test scale
 5
!
 200.000000
 400.000000
 600.000000
 800.000000
 1200.000000
";
        let scale = from_scl(original).unwrap();
        let exported = to_scl(&scale, "test", "Test scale");
        let reimported = from_scl(&exported).unwrap();

        // Values should match
        for (a, b) in scale.vector().iter().zip(reimported.vector().iter()) {
            assert!(
                (a - b).abs() < EPSILON,
                "round-trip mismatch: {} vs {}",
                a,
                b
            );
        }
    }
}

use super::{TemplateError, render, validate};

const KEYS: [(&str, usize); 7] = [
    ("day_of_week", 32),
    ("date", 10),
    ("time", 8),
    ("greeting", 14),
    ("link_status", 106),
    ("node", 63),
    ("callsign", 63),
];

fn render_text(text: &str, values: &[(&str, &str)]) -> Result<String, TemplateError> {
    let mut output = [0; 127];
    let length = render(text, values, &mut output)?;
    Ok(std::str::from_utf8(&output[..length]).unwrap().to_owned())
}

#[test]
fn accepts_only_documented_template_grammar_with_bounded_worst_case_output() {
    let largest = "x".repeat(127);
    let too_large = "x".repeat(128);
    let cases = [
        ("${day_of_week} ${date} ${time}", true),
        ("plain text", true),
        ("$x", true),
        ("", true),
        ("${link_status} ${date} ${time}x", true),
        ("${link_status} ${date} ${time}xx", false),
        ("${link_status}${greeting}${date}", false),
        ("${node}${callsign}x", true),
        ("${node}${callsign}xx", false),
        ("${unknown}", false),
        ("${xxxxxxxxxxx}", false),
        ("${xxxx}", false),
        ("${xxxxxxxx}", false),
        ("${node", false),
        ("${}", false),
        (&largest, true),
        (&too_large, false),
    ];

    for (text, valid) in cases {
        assert_eq!(
            validate(text, &KEYS, 127).is_ok(),
            valid,
            "unexpected validation result for {text:?}"
        );
    }
}

#[test]
fn renders_every_documented_substitution_literally() {
    let values = [
        ("day_of_week", "Wednesday"),
        ("date", "2026-09-10"),
        ("time", "8:15 PM"),
        ("greeting", "Good Evening"),
        ("link_status", "no links"),
        ("node", "524950"),
        ("callsign", "KG0BP"),
    ];
    let cases = [
        (
            "${day_of_week} ${date} ${time}",
            "Wednesday 2026-09-10 8:15 PM",
        ),
        ("${greeting}", "Good Evening"),
        ("${link_status}", "no links"),
        ("${node}", "524950"),
        ("${callsign}", "KG0BP"),
        ("plain text", "plain text"),
        ("$x", "$x"),
        ("", ""),
    ];

    for (text, expected) in cases {
        assert_eq!(render_text(text, &values).unwrap(), expected);
    }
}

#[test]
fn rendering_preserves_empty_values() {
    assert_eq!(render_text("${callsign}", &[("callsign", "")]).unwrap(), "");
}

#[test]
fn rendering_rejects_actual_output_overflow_without_truncation() {
    let oversized = "x".repeat(128);
    assert_eq!(
        render_text("${link_status}", &[("link_status", &oversized)]),
        Err(TemplateError::OutputTooLong)
    );
}

#[test]
fn reports_stable_template_errors() {
    assert_eq!(
        validate("${unknown}", &KEYS, 127),
        Err(TemplateError::InvalidSyntax)
    );
}

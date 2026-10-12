//! Bounded template traversal and expansion.

/// A malformed or oversized generic template.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemplateError {
    /// A placeholder is malformed or absent from the caller's keys.
    InvalidSyntax,
    /// Conservative or actual output exceeds the supplied byte limit.
    OutputTooLong,
}

/// Check a template against caller-owned placeholder names and widths.
pub fn validate(text: &str, keys: &[(&str, usize)], maximum: usize) -> Result<(), TemplateError> {
    validate_with(
        text,
        &|name| {
            keys.iter()
                .find(|(key, _)| *key == name)
                .map(|(_, width)| *width)
        },
        maximum,
    )
}

/// Expand into a caller-owned buffer without truncating an output.
pub fn render(
    text: &str,
    values: &[(&str, &str)],
    output: &mut [u8],
) -> Result<usize, TemplateError> {
    render_with(
        text,
        &|name| {
            values
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| *value)
        },
        output,
    )
}

pub(crate) fn validate_with(
    text: &str,
    width: &dyn Fn(&str) -> Option<usize>,
    maximum: usize,
) -> Result<(), TemplateError> {
    let mut total = 0;
    visit_template(text, &mut |part| {
        let added = match part {
            TemplatePart::Literal(text) => text.len(),
            TemplatePart::Substitution(name) => width(name).ok_or(TemplateError::InvalidSyntax)?,
        };
        total = checked_total(total, added, maximum)?;
        Ok(())
    })
}

pub(crate) fn render_with<'a>(
    text: &str,
    value: &dyn Fn(&str) -> Option<&'a str>,
    output: &mut [u8],
) -> Result<usize, TemplateError> {
    let mut total = 0;
    visit_template(text, &mut |part| {
        let replacement = match part {
            TemplatePart::Literal(text) => text,
            TemplatePart::Substitution(name) => value(name).ok_or(TemplateError::InvalidSyntax)?,
        };
        total = checked_total(total, replacement.len(), output.len())?;
        Ok(())
    })?;
    let mut written = 0;
    visit_template(text, &mut |part| {
        let replacement = match part {
            TemplatePart::Literal(text) => text,
            TemplatePart::Substitution(name) => value(name).ok_or(TemplateError::InvalidSyntax)?,
        };
        let end = written + replacement.len();
        output[written..end].copy_from_slice(replacement.as_bytes());
        written = end;
        Ok(())
    })?;
    Ok(total)
}

#[derive(Clone, Copy)]
enum TemplatePart<'a> {
    Literal(&'a str),
    Substitution(&'a str),
}

fn visit_template<'a>(
    text: &'a str,
    visit: &mut dyn FnMut(TemplatePart<'a>) -> Result<(), TemplateError>,
) -> Result<(), TemplateError> {
    let mut remaining = text;
    while let Some(start) = remaining.find("${") {
        visit(TemplatePart::Literal(&remaining[..start]))?;
        let substitution = &remaining[start + 2..];
        let end = substitution.find('}').ok_or(TemplateError::InvalidSyntax)?;
        visit(TemplatePart::Substitution(&substitution[..end]))?;
        remaining = &substitution[end + 1..];
    }
    visit(TemplatePart::Literal(remaining))
}

fn checked_total(used: usize, added: usize, maximum: usize) -> Result<usize, TemplateError> {
    if added > maximum - used {
        return Err(TemplateError::OutputTooLong);
    }
    Ok(used + added)
}

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn generic_template_preserves_literal_and_byte_limit_behavior() {
        let keys = [("name", 4)];
        for (text, expected) in [
            ("$x } ${name}", Ok(())),
            ("${unknown}", Err(TemplateError::InvalidSyntax)),
            ("${name", Err(TemplateError::InvalidSyntax)),
            ("${}", Err(TemplateError::InvalidSyntax)),
            ("é", Ok(())),
        ] {
            assert_eq!(validate(text, &keys, 127), expected, "{text:?}");
        }
        assert_eq!(validate(&"x".repeat(127), &keys, 127), Ok(()));
        assert_eq!(
            validate(&"x".repeat(128), &keys, 127),
            Err(TemplateError::OutputTooLong)
        );
        assert_eq!(
            validate(&"é".repeat(64), &keys, 127),
            Err(TemplateError::OutputTooLong)
        );
        let mut output = [0u8; 127];
        assert_eq!(render("$x } ${name}", &[("name", "é")], &mut output), Ok(7));
        assert_eq!(&output[..7], "$x } é".as_bytes());
        assert_eq!(
            render("${unknown}", &[("name", "é")], &mut output),
            Err(TemplateError::InvalidSyntax)
        );
        assert_eq!(render(&"x".repeat(127), &[], &mut output), Ok(127));
        assert_eq!(
            render(&"x".repeat(128), &[], &mut output),
            Err(TemplateError::OutputTooLong)
        );
        assert_eq!(
            render("${name}", &[("name", &"é".repeat(64))], &mut output),
            Err(TemplateError::OutputTooLong)
        );
    }
}

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
mod baseline_tests;

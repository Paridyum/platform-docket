//! Rendering arbitrary text safely into LaTeX source.

/// Escape the ten characters TeX treats specially.
///
/// Every value that reaches a template passes through here. An intern named
/// `O'Brien & Sons` or a project called `100% uptime` must render as written,
/// not abort the build and not inject control sequences into a legal document.
///
/// The backslash is replaced first. Replacing it later would re-escape the
/// backslashes introduced by the earlier replacements.
#[must_use]
pub fn tex(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str(r"\textbackslash{}"),
            '&' => out.push_str(r"\&"),
            '%' => out.push_str(r"\%"),
            '$' => out.push_str(r"\$"),
            '#' => out.push_str(r"\#"),
            '_' => out.push_str(r"\_"),
            '{' => out.push_str(r"\{"),
            '}' => out.push_str(r"\}"),
            '~' => out.push_str(r"\textasciitilde{}"),
            '^' => out.push_str(r"\textasciicircum{}"),
            '"' => out.push_str("''"),
            other => out.push(other),
        }
    }
    out
}

/// Convert an arbitrary key into a TeX command name.
///
/// TeX command names are letters only, so `pre_existing_ip` becomes
/// `PreExistingIp`. Digits are kept but a name is never allowed to start
/// with one, because `\dk1Foo` does not parse.
#[must_use]
pub fn command_name(key: &str) -> String {
    let mut out = String::with_capacity(key.len());
    let mut capitalise = true;
    for ch in key.chars() {
        if ch.is_ascii_alphanumeric() {
            if capitalise {
                out.extend(ch.to_uppercase());
                capitalise = false;
            } else {
                out.push(ch);
            }
        } else {
            capitalise = true;
        }
    }
    out
}

#[cfg(test)]
// A test that cannot assert an expected value is not a test. The lint stays
// denied for the crate; only assertions inside test modules are exempt.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::{command_name, tex};

    #[test]
    fn escapes_every_special_character() {
        assert_eq!(
            tex("a&b%c$d#e_f{g}h~i^j"),
            r"a\&b\%c\$d\#e\_f\{g\}h\textasciitilde{}i\textasciicircum{}j"
        );
    }

    #[test]
    fn backslash_is_not_re_escaped() {
        // The failure this guards: replacing `\` after `&` turns `\&` into
        // `\textbackslash{}&`, which renders as literal text in a contract.
        assert_eq!(tex(r"a\b"), r"a\textbackslash{}b");
        assert_eq!(tex(r"\&"), r"\textbackslash{}\&");
    }

    #[test]
    fn ordinary_names_pass_through() {
        assert_eq!(tex("Asha Verma"), "Asha Verma");
        assert_eq!(tex("O'Brien & Sons"), r"O'Brien \& Sons");
    }

    #[test]
    fn command_names_are_camel_case() {
        assert_eq!(command_name("pre_existing_ip"), "PreExistingIp");
        assert_eq!(command_name("legal-name"), "LegalName");
        assert_eq!(command_name("org"), "Org");
    }
}

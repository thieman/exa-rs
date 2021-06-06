use regex::Regex;

fn strip_whitespace(i: &str) -> String {
    let leading = Regex::new(r"(?m)^\s+").unwrap();
    let replaced = leading.replace_all(i, "");

    let trailing = Regex::new(r"(?m)[[:blank:]]+$").unwrap();
    trailing.replace_all(&replaced, "").to_owned().to_string()
}

fn strip_comments(i: &str) -> String {
    let inline = Regex::new(r"(?m);.*$").unwrap();
    let replaced = inline.replace_all(i, "");

    let note = Regex::new(r"(?mi)note.*$").unwrap();
    note.replace_all(&replaced, "").to_owned().to_string()
}

fn strip_empty_lines(i: &str) -> String {
    let empty = Regex::new(r"(?ms)^[[:blank:]]*\n").unwrap();
    empty.replace_all(i, "").to_owned().to_string()
}

pub fn preprocess_text(i: &str) -> String {
    let mut out = strip_whitespace(i);
    out = strip_comments(&out);
    out = strip_empty_lines(&out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_whitespace() {
        assert_eq!(strip_whitespace(" t "), String::from("t"));
        assert_eq!(
            strip_whitespace(" \t\tcontent ok   \t"),
            String::from("content ok")
        );
        assert_eq!(
            strip_whitespace(" line 1\n\tline two   \nline three"),
            String::from("line 1\nline two\nline three")
        );
    }

    #[test]
    fn test_strip_comments() {
        assert_eq!(
            strip_comments("nothing to see here\n"),
            String::from("nothing to see here\n")
        );
        assert_eq!(
            strip_comments("nothing to ;see here\n"),
            String::from("nothing to \n")
        );
        assert_eq!(
            strip_comments("nothing to;;;see here\n"),
            String::from("nothing to\n")
        );
        assert_eq!(strip_comments("; full line comment \n"), String::from("\n"));
        assert_eq!(
            strip_comments("NOTE nothing to see here \n"),
            String::from("\n")
        );
        assert_eq!(
            strip_comments("can also note inline\n"),
            String::from("can also \n")
        );
    }

    #[test]
    fn test_strip_empty_lines() {
        assert_eq!(
            strip_empty_lines("  some bullshit\n\n\n  ok\n"),
            String::from("  some bullshit\n  ok\n")
        );
    }
}

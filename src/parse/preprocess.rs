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

fn expand_macros(i: &str) -> String {
    let macros =
        Regex::new(r"(?is)@rep[[:blank:]]+(\d+)[[:blank:]]?\n(.+?)[[:blank:]]?@end[[:blank:]]?\n")
            .unwrap();
    let expansions = Regex::new(r"@\{(-?\d+),(-?\d+)\}").unwrap();

    let mut out = String::with_capacity(i.len());

    let mut furthest_read: usize = 0;

    for caps in macros.captures_iter(i) {
        out.push_str(&i[furthest_read..caps.get(0).unwrap().start()]);

        let reps: i16 = caps.get(1).unwrap().as_str().parse().unwrap();

        let raw_body = caps.get(2).unwrap().as_str();
        let raw_expansions: Vec<(&str, i16, i16)> = expansions
            .captures_iter(raw_body)
            .map(|c| {
                (
                    c.get(0).unwrap().as_str(),
                    c.get(1).unwrap().as_str().parse().unwrap(),
                    c.get(2).unwrap().as_str().parse().unwrap(),
                )
            })
            .collect();

        for iteration in 0..reps {
            let mut replaced_body = raw_body.to_string();
            for exp in &raw_expansions {
                let value = exp.1 + (iteration * exp.2);
                replaced_body = replaced_body.replace(exp.0, &value.to_string());
            }

            out.push_str(&replaced_body);
        }

        furthest_read = caps.get(0).unwrap().end();
    }

    // Add everything past the last macro match
    out.push_str(&i[furthest_read..]);
    out
}

pub fn preprocess_text(i: &str) -> String {
    let mut out = strip_whitespace(i);
    out = strip_comments(&out);
    out = strip_empty_lines(&out);
    expand_macros(&out)
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

    #[test]
    fn test_expand_macros() {
        assert_eq!(
            expand_macros("nothing\nto expand\n"),
            String::from("nothing\nto expand\n"),
        );

        assert_eq!(
            expand_macros("header\n@rep 5\nbooya\n@end\n"),
            String::from("header\nbooya\nbooya\nbooya\nbooya\nbooya\n"),
        );

        assert_eq!(
            expand_macros("@rep 2\nlink @{3,5}\ncopy @{1,2} x\n@end\n"),
            String::from("link 3\ncopy 1 x\nlink 8\ncopy 3 x\n"),
        );

        assert_eq!(
            expand_macros("@rep 2\nlink @{-1,-4}\ncopy @{-5,3} x\n@end\n"),
            String::from("link -1\ncopy -5 x\nlink -5\ncopy -2 x\n"),
        );
    }

    #[test]
    fn test_multiple_macros() {
        assert_eq!(
            expand_macros("@rep 2\nnoop\n@end\n@rep 2\ncopy 1 x\n@end\n"),
            String::from("noop\nnoop\ncopy 1 x\ncopy 1 x\n"),
        )
    }

    #[test]
    fn test_macro_expand_value_multiple_digits() {
        assert_eq!(
            expand_macros("@rep 2\ncopy @{20,40} x\n @end\n"),
            String::from("copy 20 x\ncopy 60 x\n"),
        );
    }
}

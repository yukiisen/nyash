enum State {
    Normal,
    InQuotes,
    InDoubleQuotes,
    Escape(bool)
}


pub fn parse_args (cmd: &str) -> Vec<String> {
    let mut tokens = Vec::new(); 
    let mut arg = String::new();
    let mut state = State::Normal;

    let cmd = cmd.replace("\n", " ");

    for char in cmd.chars() {
        match state {
            State::Normal => {
                match char {
                    '\'' => {
                        state = State::InQuotes;
                    }
                    '"' => {
                        state = State::InDoubleQuotes;
                    }
                    ' ' => {
                        if arg.is_empty() { continue; }
                        tokens.push(arg);
                        arg = String::new();
                    }
                    '\\' => {
                        state = State::Escape(false);
                    }
                    ch => {
                        arg.push(ch);
                    }
                }
            },
            State::InQuotes => {
                match char {
                    '\'' => {
                        state = State::Normal;
                    },
                    ch => {
                        arg.push(ch);
                    }
                }
            },
            State::InDoubleQuotes => {
                match char {
                    '"' => {
                        state = State::Normal;
                    }
                    '\\' => {
                        state = State::Escape(true);
                    }
                    ch => {
                        arg.push(ch);
                    }
                }
            },
            State::Escape(quote) => {
                state = if quote { State::InDoubleQuotes } else { State::Normal };

                if !['\\', '"', '$', '`'].contains(&char) && quote {
                    arg.push('\\');
                }

                arg.push(char);
            }
        }
    };

    if !arg.is_empty() { tokens.push(arg); };

    tokens
}

#[cfg(test)]
mod tests {
    use super::parse_args;

    #[test]
    fn test_basic() {
        assert_eq!(
            parse_args("echo hello world"),
            vec!["echo", "hello", "world"]
        );
    }

    #[test]
    fn test_single_quotes() {
        assert_eq!(
            parse_args("echo 'hello world' foo"),
            vec!["echo", "hello world", "foo"]
        );
    }

    #[test]
    fn test_only_single_quoted() {
        assert_eq!(
            parse_args("'just one quoted arg'"),
            vec!["just one quoted arg"]
        );
    }

    #[test]
    fn test_mixed_quoted_and_unquoted() {
        assert_eq!(
            parse_args("cmd 'arg with space' another"),
            vec!["cmd", "arg with space", "another"]
        );
    }

    #[test]
    fn test_multiple_spaces() {
        assert_eq!(
            parse_args("   echo    foo   bar "),
            vec!["echo", "foo", "bar"]
        );
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(
            parse_args(""),
            Vec::<String>::new()
        );
    }

    #[test]
    fn test_only_spaces() {
        assert_eq!(
            parse_args("     "),
            Vec::<String>::new()
        );
    }

    #[test]
    fn test_unclosed_quote() {
        // this will depend on your current implementation — this assumes you just keep collecting
        assert_eq!(
            parse_args("echo 'unclosed quote"),
            vec!["echo", "unclosed quote"]
        );
    }

    #[test]
    fn test_nested_single_quotes_ignored() {
        // no quote escaping supported yet
        assert_eq!(
            parse_args("echo 'it is 'not' nested'"),
            vec!["echo", "it is not nested"]
        );
    }

    #[test]
    fn test_complex_paths() {
        let input = "cat \"/tmp/'file name' with spaces\"";
        let args = parse_args(input);
        assert_eq!(args, vec!["cat", "/tmp/'file name' with spaces"]);
    }

    #[test]
    fn test_backslashes_in_double_quotes() {
        let input = r#"echo "a\\b" "a\"b" "a\$b" "a\`b" "a\zb""#;
        let parsed = parse_args(input);

        assert_eq!(
            parsed,
            vec![
                "echo".to_string(),
                "a\\b".to_string(),
                "a\"b".to_string(),
                "a$b".to_string(),
                "a`b".to_string(),
                "a\\zb".to_string(),
            ]
        );
    }
}

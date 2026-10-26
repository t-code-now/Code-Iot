//! A tiny syntax highlighter: good enough for lesson snippets, no deps.
//! Pure Rust, so it is unit-tested natively.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Plain,
    Keyword,
    Str,
    Num,
    Comment,
    Tag,
    Func,
}

impl Kind {
    pub fn class(self) -> &'static str {
        match self {
            Kind::Plain => "tk-plain",
            Kind::Keyword => "tk-kw",
            Kind::Str => "tk-str",
            Kind::Num => "tk-num",
            Kind::Comment => "tk-com",
            Kind::Tag => "tk-tag",
            Kind::Func => "tk-fn",
        }
    }
}

fn keywords(lang: &str) -> &'static [&'static str] {
    match lang {
        "python" => &[
            "def", "return", "if", "elif", "else", "for", "while", "in", "and", "or", "not", "import",
            "from", "class", "True", "False", "None", "print", "range", "len", "break", "continue",
        ],
        "javascript" => &[
            "const", "let", "var", "function", "return", "if", "else", "for", "while", "of", "in",
            "true", "false", "null", "undefined", "new", "class", "console",
        ],
        "sql" => &[
            "SELECT", "FROM", "WHERE", "ORDER", "BY", "GROUP", "LIMIT", "INSERT", "INTO", "VALUES",
            "UPDATE", "SET", "DELETE", "AND", "OR", "NOT", "AS", "DESC", "ASC", "COUNT", "SUM", "AVG",
            "JOIN", "ON",
        ],
        "rust" => &[
            "fn", "let", "mut", "match", "if", "else", "for", "while", "loop", "return", "struct",
            "enum", "impl", "use", "pub", "Some", "None", "Ok", "Err", "String", "true", "false",
        ],
        "arduino" => &[
            "void", "int", "long", "float", "double", "bool", "byte", "char", "unsigned", "const",
            "volatile", "static", "if", "else", "for", "while", "do", "return", "switch", "case",
            "break", "default", "enum", "class", "struct", "public", "private", "true", "false",
            "include", "define", "HIGH", "LOW", "INPUT", "OUTPUT", "INPUT_PULLUP", "LED_BUILTIN",
            "word", "String", "protected", "virtual", "override", "template", "typename", "typedef",
            "using", "constexpr", "nullptr", "sizeof", "uint8_t", "int16_t", "uint32_t", "PROGMEM",
            "continue", "pragma", "ifdef", "endif",
        ],
        "cpp" => &[
            "int", "long", "short", "float", "double", "bool", "char", "void", "auto", "unsigned",
            "signed", "const", "constexpr", "static", "if", "else", "for", "while", "do", "return",
            "switch", "case", "break", "continue", "default", "class", "struct", "enum", "public",
            "private", "protected", "virtual", "override", "template", "typename", "using",
            "namespace", "new", "delete", "nullptr", "true", "false", "this", "try", "catch",
            "throw", "include", "std", "string", "vector", "cout", "cin", "endl", "size_t", "explicit",
            "operator", "friend", "noexcept", "sizeof", "typedef", "inline", "mutable", "final",
        ],
        "css" | "html" => &[
            "color", "display", "flex", "margin", "padding", "border", "background", "font-size",
            "justify-content", "align-items", "flex-direction", "solid",
        ],
        _ => &[],
    }
}

fn line_comment(lang: &str) -> &'static [&'static str] {
    match lang {
        "python" => &["#"],
        "sql" => &["--"],
        "javascript" | "rust" | "css" | "arduino" | "cpp" => &["//"],
        _ => &[],
    }
}

/// Split `code` into highlighted pieces. Concatenating the texts gives back
/// the original code exactly.
pub fn tokens(code: &str, lang: &str) -> Vec<(Kind, String)> {
    let chars: Vec<char> = code.chars().collect();
    let kws = keywords(lang);
    let comments = line_comment(lang);
    let is_markup = lang == "html";
    let mut out: Vec<(Kind, String)> = Vec::new();
    let mut i = 0;

    let push = |out: &mut Vec<(Kind, String)>, kind: Kind, text: String| {
        if let Some(last) = out.last_mut() {
            if last.0 == kind && kind == Kind::Plain {
                last.1.push_str(&text);
                return;
            }
        }
        out.push((kind, text));
    };

    while i < chars.len() {
        let c = chars[i];
        let rest: String = chars[i..].iter().take(3).collect();

        // line comments
        if let Some(_) = comments.iter().find(|m| rest.starts_with(**m)) {
            let end = chars[i..].iter().position(|&ch| ch == '\n').map_or(chars.len(), |p| i + p);
            push(&mut out, Kind::Comment, chars[i..end].iter().collect());
            i = end;
            continue;
        }
        // HTML tags: <name ...> or </name>
        if is_markup && c == '<' {
            let end = chars[i..].iter().position(|&ch| ch == '>').map_or(chars.len(), |p| i + p + 1);
            push(&mut out, Kind::Tag, chars[i..end].iter().collect());
            i = end;
            continue;
        }
        // strings
        if c == '"' || c == '\'' || c == '`' {
            let mut j = i + 1;
            while j < chars.len() && chars[j] != c && chars[j] != '\n' {
                if chars[j] == '\\' {
                    j += 1;
                }
                j += 1;
            }
            let end = (j + 1).min(chars.len());
            push(&mut out, Kind::Str, chars[i..end].iter().collect());
            i = end;
            continue;
        }
        // numbers
        if c.is_ascii_digit() {
            let mut j = i;
            while j < chars.len() && (chars[j].is_ascii_digit() || chars[j] == '.') {
                j += 1;
            }
            push(&mut out, Kind::Num, chars[i..j].iter().collect());
            i = j;
            continue;
        }
        // words
        if c.is_alphabetic() || c == '_' {
            let mut j = i;
            while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_' || chars[j] == '-' && is_css_word(lang)) {
                j += 1;
            }
            let word: String = chars[i..j].iter().collect();
            let kind = if kws.contains(&word.as_str()) {
                Kind::Keyword
            } else if chars.get(j) == Some(&'(') || chars.get(j) == Some(&'!') {
                Kind::Func
            } else {
                Kind::Plain
            };
            push(&mut out, kind, word);
            i = j;
            continue;
        }
        push(&mut out, Kind::Plain, c.to_string());
        i += 1;
    }
    out
}

fn is_css_word(lang: &str) -> bool {
    lang == "css" || lang == "html"
}

/// The highlighter language for a course id.
pub fn lang_for_course(course_id: &str) -> &'static str {
    match course_id {
        "python" => "python",
        "javascript" => "javascript",
        "web" => "html",
        "sql" => "sql",
        "rust" => "rust",
        "arduino" => "arduino",
        "cpp" => "cpp",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn joined(code: &str, lang: &str) -> String {
        tokens(code, lang).into_iter().map(|t| t.1).collect()
    }

    #[test]
    fn round_trips() {
        for (code, lang) in [
            ("def f(x):\n    return x * 2  # double", "python"),
            ("const s = `hi ${n}`; // c", "javascript"),
            ("<p class=\"a\">Hi</p>", "html"),
            ("SELECT * FROM t WHERE a = 'x';", "sql"),
            ("let mut y = \"unterminated", "rust"),
            ("#include <Servo.h>\nvoid setup() { pinMode(13, OUTPUT); } // x", "arduino"),
        ] {
            assert_eq!(joined(code, lang), code);
        }
    }

    #[test]
    fn kinds() {
        let t = tokens("print(\"hi\") # c", "python");
        assert_eq!(t[0], (Kind::Keyword, "print".into()));
        assert!(t.contains(&(Kind::Str, "\"hi\"".into())));
        assert_eq!(t.last().unwrap(), &(Kind::Comment, "# c".into()));
        let r = tokens("println!(\"x\")", "rust");
        assert_eq!(r[0], (Kind::Func, "println".into()));
        let n = tokens("x = 42", "python");
        assert!(n.contains(&(Kind::Num, "42".into())));
    }
}

//! Colouring Python source for the source panel: the code cut into runs, each
//! with the kind of thing it is. The runs put back together are the code, byte
//! for byte, so a screen reader and a copy get the plain text.

/// What a run of source is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Everything else: names, operators, brackets, spaces.
    Plain,
    /// `def`, `return`, `None` and the other reserved words.
    Keyword,
    /// A string, with its prefix and quotes; a docstring is one.
    String,
    /// From `#` to the end of the line.
    Comment,
    /// A number: `28`, `0.05`, `1e-5`, `0xff`.
    Number,
    /// A decorator (`@element_51.node`), or the name after `def` or `class`.
    Name,
}

impl Kind {
    /// The class name the page gives a run of this kind, or None for plain text.
    pub fn class(self) -> Option<&'static str> {
        match self {
            Kind::Plain => None,
            Kind::Keyword => Some("tok-keyword"),
            Kind::String => Some("tok-string"),
            Kind::Comment => Some("tok-comment"),
            Kind::Number => Some("tok-number"),
            Kind::Name => Some("tok-name"),
        }
    }
}

/// A run of source and its kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub kind: Kind,
    pub text: String,
}

const KEYWORDS: [&str; 35] = [
    "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class", "continue",
    "def", "del", "elif", "else", "except", "finally", "for", "from", "global", "if", "import",
    "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try", "while",
    "with", "yield",
];

/// Letters that may come before a string's quote: `r`, `b`, `f`, `u` and the
/// pairs `rb`, `br`, `rf`, `fr`, in either case.
fn is_string_prefix(word: &str) -> bool {
    let lower = word.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "r" | "b" | "f" | "u" | "rb" | "br" | "rf" | "fr"
    )
}

fn is_word_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The byte offset in `code` where the string whose first quote is at byte
/// `quote` ends. A string that is not closed runs to the end of its line, or
/// for a triple quote to the end of the code, so half-typed code still shows
/// something sensible.
fn string_end(code: &str, quote: usize) -> usize {
    let rest = &code[quote..];
    let mark = rest.chars().next().expect("the caller saw a quote");
    let triple: String = std::iter::repeat_n(mark, 3).collect();
    let mut chars = rest.char_indices();
    if rest.starts_with(&triple) {
        let mut at = triple.len();
        while at < rest.len() {
            if rest[at..].starts_with('\\') {
                at += 1 + rest[at + 1..].chars().next().map_or(0, char::len_utf8);
            } else if rest[at..].starts_with(&triple) {
                return quote + at + triple.len();
            } else {
                at += rest[at..].chars().next().map_or(1, char::len_utf8);
            }
        }
        return code.len();
    }
    chars.next();
    while let Some((at, c)) = chars.next() {
        match c {
            '\\' => {
                chars.next();
            }
            '\n' => return quote + at,
            c if c == mark => return quote + at + c.len_utf8(),
            _ => {}
        }
    }
    code.len()
}

/// The byte offset in `code` where the number that starts at byte `from` ends.
fn number_end(code: &str, from: usize) -> usize {
    let rest = &code[from..];
    let hex = rest.starts_with("0x") || rest.starts_with("0X");
    let mut previous = ' ';
    let mut end = 0;
    for (at, c) in rest.char_indices() {
        let sign_of_exponent = !hex && matches!(c, '+' | '-') && matches!(previous, 'e' | 'E');
        if !(c.is_ascii_alphanumeric() || c == '_' || c == '.' || sign_of_exponent) {
            break;
        }
        previous = c;
        end = at + c.len_utf8();
    }
    from + end
}

/// Cut Python `code` into runs. Every byte is in exactly one run, in order, and
/// neighbouring plain text is one run.
pub fn highlight(code: &str) -> Vec<Span> {
    let mut spans: Vec<Span> = Vec::new();
    let mut push = |kind: Kind, text: &str| {
        if text.is_empty() {
            return;
        }
        if kind == Kind::Plain
            && let Some(last) = spans.last_mut()
            && last.kind == Kind::Plain
        {
            last.text.push_str(text);
            return;
        }
        spans.push(Span {
            kind,
            text: text.to_string(),
        });
    };
    let mut at = 0;
    let mut name_next = false;
    while at < code.len() {
        let rest = &code[at..];
        let c = rest.chars().next().expect("at is inside the code");
        let end = if c == '#' {
            let end = rest.find('\n').map_or(code.len(), |line| at + line);
            push(Kind::Comment, &code[at..end]);
            end
        } else if c == '"' || c == '\'' {
            let end = string_end(code, at);
            push(Kind::String, &code[at..end]);
            end
        } else if c.is_ascii_digit()
            || (c == '.' && rest[1..].starts_with(|d: char| d.is_ascii_digit()))
        {
            let end = number_end(code, at);
            push(Kind::Number, &code[at..end]);
            end
        } else if c == '@' && rest[1..].starts_with(is_word_start) {
            let length = rest[1..]
                .find(|d: char| !(is_word_char(d) || d == '.'))
                .map_or(rest.len(), |stop| stop + 1);
            push(Kind::Name, &rest[..length]);
            at + length
        } else if is_word_start(c) {
            let length = rest.find(|d: char| !is_word_char(d)).unwrap_or(rest.len());
            let word = &rest[..length];
            let quote = rest[length..].chars().next();
            if matches!(quote, Some('"' | '\'')) && is_string_prefix(word) {
                let end = string_end(code, at + length);
                push(Kind::String, &code[at..end]);
                end
            } else {
                let kind = if name_next {
                    Kind::Name
                } else if KEYWORDS.contains(&word) {
                    Kind::Keyword
                } else {
                    Kind::Plain
                };
                name_next = matches!(word, "def" | "class") && kind == Kind::Keyword;
                push(kind, word);
                at + length
            }
        } else {
            push(Kind::Plain, &rest[..c.len_utf8()]);
            at + c.len_utf8()
        };
        // Spaces between `def` and the name do not end the wait for the name.
        if !(c == ' ' && name_next) && !is_word_start(c) {
            name_next = false;
        }
        at = end;
    }
    spans
}

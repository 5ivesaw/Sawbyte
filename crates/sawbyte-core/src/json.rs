//! Minimal, zero-dependency RFC 8259 compliant JSON value and parser.

#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    pub fn parse(input: &str) -> Result<Self, String> {
        let mut chars = input.char_indices().peekable();
        skip_ws(&mut chars);
        let val = parse_val(&mut chars, input)?;
        skip_ws(&mut chars);
        if let Some((idx, _)) = chars.peek() {
            return Err(format!("unexpected trailing character at byte {idx}"));
        }
        Ok(val)
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            Self::Object(pairs) => {
                for (k, v) in pairs {
                    if k == key {
                        return Some(v);
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Number(n) => Some(*n as i64),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Self::Number(n) if *n >= 0.0 => Some(*n as u64),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[JsonValue]> {
        match self {
            Self::Array(arr) => Some(arr.as_slice()),
            _ => None,
        }
    }

    pub fn to_string_lossy(&self) -> String {
        match self {
            Self::Null => "null".to_string(),
            Self::Bool(b) => b.to_string(),
            Self::Number(n) => {
                if n.fract() == 0.0 && n.is_finite() && *n >= (i64::MIN as f64) && *n <= (i64::MAX as f64) {
                    format!("{}", *n as i64)
                } else {
                    format!("{n}")
                }
            }
            Self::String(s) => s.clone(),
            Self::Array(arr) => {
                let inner: Vec<String> = arr.iter().map(|v| v.to_string_lossy()).collect();
                format!("[{}]", inner.join(", "))
            }
            Self::Object(pairs) => {
                let inner: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("\"{k}\": {}", v.to_string_lossy()))
                    .collect();
                format!("{{{}}}", inner.join(", "))
            }
        }
    }
}

type CharIter<'a> = std::iter::Peekable<std::str::CharIndices<'a>>;

fn skip_ws(chars: &mut CharIter<'_>) {
    while let Some(&(_, c)) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}

fn parse_val(chars: &mut CharIter<'_>, input: &str) -> Result<JsonValue, String> {
    skip_ws(chars);
    let &(_, c) = chars.peek().ok_or_else(|| "unexpected end of JSON input".to_string())?;
    match c {
        'n' => parse_literal(chars, "null", JsonValue::Null),
        't' => parse_literal(chars, "true", JsonValue::Bool(true)),
        'f' => parse_literal(chars, "false", JsonValue::Bool(false)),
        '"' => parse_string(chars).map(JsonValue::String),
        '[' => parse_array(chars, input),
        '{' => parse_object(chars, input),
        '-' | '0'..='9' => parse_number(chars, input),
        other => Err(format!("unexpected character '{other}' in JSON input")),
    }
}

fn parse_literal(chars: &mut CharIter<'_>, expected: &str, val: JsonValue) -> Result<JsonValue, String> {
    for exp_c in expected.chars() {
        match chars.next() {
            Some((_, c)) if c == exp_c => {}
            _ => return Err(format!("expected literal '{expected}'")),
        }
    }
    Ok(val)
}

fn parse_string(chars: &mut CharIter<'_>) -> Result<String, String> {
    chars.next(); // consume opening '"'
    let mut out = String::new();
    while let Some((_, c)) = chars.next() {
        match c {
            '"' => return Ok(out),
            '\\' => {
                let (_, esc) = chars.next().ok_or_else(|| "unterminated escape sequence in string".to_string())?;
                match esc {
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    '/' => out.push('/'),
                    'b' => out.push('\u{0008}'),
                    'f' => out.push('\u{000c}'),
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    'u' => {
                        let mut hex = String::with_capacity(4);
                        for _ in 0..4 {
                            let (_, hc) = chars.next().ok_or_else(|| "incomplete \\u hex escape".to_string())?;
                            hex.push(hc);
                        }
                        let code = u32::from_str_radix(&hex, 16)
                            .map_err(|_| format!("invalid unicode escape \\u{hex}"))?;
                        let ch = char::from_u32(code)
                            .ok_or_else(|| format!("invalid unicode codepoint \\u{hex}"))?;
                        out.push(ch);
                    }
                    other => return Err(format!("invalid escape character '\\{other}'")),
                }
            }
            c => out.push(c),
        }
    }
    Err("unterminated string literal in JSON".to_string())
}

fn parse_array(chars: &mut CharIter<'_>, input: &str) -> Result<JsonValue, String> {
    chars.next(); // consume '['
    skip_ws(chars);
    if let Some(&(_, ']')) = chars.peek() {
        chars.next();
        return Ok(JsonValue::Array(Vec::new()));
    }
    let mut elements = Vec::new();
    loop {
        skip_ws(chars);
        elements.push(parse_val(chars, input)?);
        skip_ws(chars);
        match chars.next() {
            Some((_, ']')) => break,
            Some((_, ',')) => continue,
            Some((idx, other)) => return Err(format!("expected ',' or ']' at byte {idx}, found '{other}'")),
            None => return Err("unexpected end of array".to_string()),
        }
    }
    Ok(JsonValue::Array(elements))
}

fn parse_object(chars: &mut CharIter<'_>, input: &str) -> Result<JsonValue, String> {
    chars.next(); // consume '{'
    skip_ws(chars);
    if let Some(&(_, '}')) = chars.peek() {
        chars.next();
        return Ok(JsonValue::Object(Vec::new()));
    }
    let mut pairs = Vec::new();
    loop {
        skip_ws(chars);
        let key = match chars.peek() {
            Some(&(_, '"')) => parse_string(chars)?,
            Some((idx, other)) => return Err(format!("expected object string key at byte {idx}, found '{other}'")),
            None => return Err("unexpected end of object key".to_string()),
        };
        skip_ws(chars);
        match chars.next() {
            Some((_, ':')) => {}
            Some((idx, other)) => return Err(format!("expected ':' after key at byte {idx}, found '{other}'")),
            None => return Err("unexpected end of object after key".to_string()),
        }
        skip_ws(chars);
        let val = parse_val(chars, input)?;
        pairs.push((key, val));
        skip_ws(chars);
        match chars.next() {
            Some((_, '}')) => break,
            Some((_, ',')) => continue,
            Some((idx, other)) => return Err(format!("expected ',' or '}}' at byte {idx}, found '{other}'")),
            None => return Err("unexpected end of object".to_string()),
        }
    }
    Ok(JsonValue::Object(pairs))
}

fn parse_number(chars: &mut CharIter<'_>, input: &str) -> Result<JsonValue, String> {
    let start = chars.peek().map(|&(i, _)| i).unwrap_or(0);
    let mut end = start;
    while let Some(&(i, c)) = chars.peek() {
        if c.is_ascii_digit() || c == '-' || c == '+' || c == '.' || c == 'e' || c == 'E' {
            end = i + c.len_utf8();
            chars.next();
        } else {
            break;
        }
    }
    let slice = &input[start..end];
    let num = slice.parse::<f64>().map_err(|_| format!("invalid JSON number '{slice}'"))?;
    Ok(JsonValue::Number(num))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_json_basic_types() {
        let v = JsonValue::parse(r#"{"command": "task.navigate", "args": [3, -60, 2], "ok": true}"#).unwrap();
        assert_eq!(v.get("command").and_then(|v| v.as_str()), Some("task.navigate"));
        assert_eq!(v.get("ok").and_then(|v| v.as_bool()), Some(true));
        let args = v.get("args").and_then(|v| v.as_array()).unwrap();
        assert_eq!(args.len(), 3);
        assert_eq!(args[0].as_i64(), Some(3));
        assert_eq!(args[1].as_i64(), Some(-60));
        assert_eq!(args[2].as_i64(), Some(2));
    }

    #[test]
    fn parse_json_escaped_string() {
        let v = JsonValue::parse(r#"{"msg": "Hello\nWorld \"quoted\" \\ \u0041"}"#).unwrap();
        assert_eq!(v.get("msg").and_then(|v| v.as_str()), Some("Hello\nWorld \"quoted\" \\ A"));
    }
}

//! Turns an Arduino sketch into a complete C++ program for the simulator,
//! the same way the Arduino IDE does: library includes are provided by the
//! simulator, function prototypes are generated (so functions may be used
//! before they are defined), and a `main` runs `setup()` then `loop()`.

pub const ARDUINO_SIM: &str = include_str!("arduino_sim.hpp");

/// Headers the simulator already provides.
const PROVIDED: &[&str] = &[
    "Arduino.h", "Servo.h", "Wire.h", "LiquidCrystal_I2C.h", "DHT.h", "EEPROM.h", "SPI.h",
    "avr/pgmspace.h", "math.h", "stdlib.h", "string.h",
];

const NOT_FUNCTIONS: &[&str] = &[
    "if", "for", "while", "switch", "else", "return", "do", "class", "struct", "enum", "union",
    "namespace", "typedef", "template", "using", "case", "default",
];

fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// If `line` starts a top-level function definition, returns its prototype.
fn prototype(line: &str, prev: &str) -> Option<String> {
    if line.starts_with(char::is_whitespace) || line.starts_with('#') || line.starts_with("//") {
        return None;
    }
    if prev.trim_start().starts_with("template") {
        return None;
    }
    let open = line.find('(')?;
    let head = line[..open].trim();
    if head.contains('=') || head.contains("::") || head.contains('"') {
        return None;
    }
    let first = head.split_whitespace().next()?;
    if NOT_FUNCTIONS.contains(&first) {
        return None;
    }
    let name = head.rsplit(|c: char| c.is_whitespace() || c == '*' || c == '&').next()?;
    if !is_ident(name) || head.len() == name.len() {
        return None; // needs a return type and a name
    }
    // The parameter list must close on this line and be followed by `{`.
    let mut depth = 0;
    let mut close = None;
    for (i, c) in line[open..].char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(open + i);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = close?;
    let params = &line[open + 1..close];
    if params.contains('=') {
        return None; // default arguments may only appear once
    }
    let rest = line[close + 1..].trim();
    let rest = rest.split("//").next().unwrap_or("").trim();
    if !(rest.starts_with('{') || rest.is_empty() || rest == "const {") {
        return None;
    }
    Some(format!("{};", line[..=close].trim()))
}

/// Builds the program for an Arduino sketch.
pub fn arduino_program(sketch: &str) -> String {
    let lines: Vec<&str> = sketch.lines().collect();
    let mut body = Vec::with_capacity(lines.len());
    let mut protos = Vec::new();
    let mut first_fn = None;
    let mut prev = "";
    for (i, raw) in lines.iter().enumerate() {
        let trimmed = raw.trim();
        let provided = trimmed.starts_with("#include")
            && PROVIDED.iter().any(|h| trimmed.contains(&format!("<{h}>")) || trimmed.contains(&format!("\"{h}\"")));
        body.push(if provided { String::new() } else { raw.to_string() });
        let next_is_brace = lines.get(i + 1).is_some_and(|n| n.trim_start().starts_with('{'));
        let ends_ok = trimmed.ends_with('{') || (trimmed.ends_with(')') && next_is_brace);
        if ends_ok {
            if let Some(p) = prototype(raw, prev) {
                first_fn.get_or_insert(i);
                protos.push(p);
            }
        }
        if !trimmed.is_empty() {
            prev = raw;
        }
    }

    let mut out = String::with_capacity(ARDUINO_SIM.len() + sketch.len() + 512);
    out.push_str(ARDUINO_SIM);
    out.push_str("\n#line 1 \"sketch.ino\"\n");
    for (i, line) in body.iter().enumerate() {
        if Some(i) == first_fn {
            for p in &protos {
                out.push_str(p);
                out.push('\n');
            }
            out.push_str(&format!("#line {} \"sketch.ino\"\n", i + 1));
        }
        out.push_str(line);
        out.push('\n');
    }
    out.push_str("\n#line 1 \"simulator\"\nint main() { setup(); for (;;) { loop(); __advance(10); } }\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_prototypes() {
        assert_eq!(prototype("void blink(int pin, int ms) {", "").as_deref(), Some("void blink(int pin, int ms);"));
        assert_eq!(prototype("const Sensor *findHighest(const Sensor *list, int n) {", "").as_deref(),
                   Some("const Sensor *findHighest(const Sensor *list, int n);"));
        assert_eq!(prototype("  if (x) {", ""), None);
        assert_eq!(prototype("if (x) {", ""), None);
        assert_eq!(prototype("T keepIn(T v, T lo, T hi) {", "template <typename T>"), None);
        assert_eq!(prototype("void Foo::bar() {", ""), None);
        assert_eq!(prototype("void f(int a = 3) {", ""), None);
        assert_eq!(prototype("class Effect {", ""), None);
    }

    #[test]
    fn strips_provided_includes_and_adds_main() {
        let p = arduino_program("#include <Servo.h>\nvoid setup() {\n}\nvoid loop() {\n  go();\n}\nvoid go() {\n}\n");
        assert!(p.contains("void go();"));
        assert!(!p.contains("#include <Servo.h>"));
        assert!(p.contains("int main()"));
    }
}

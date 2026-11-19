//! Double-precision NumberBox state, decimal formatting, and expression evaluation.

/// Numeric parsing and formatting can be supplied by a host's regional formatter.
pub trait NumberFormatter {
    /// Parses one complete numeric token. `None` denotes invalid syntax.
    fn parse(&self, text: &str) -> Option<f64>;
    /// Formats the current value for display; a formatter owns its display rounding.
    /// NaN is represented as empty text.
    fn format(&self, value: f64) -> String;
}

/// Portable decimal formatter; grouping/currency are left to a host formatter.
#[derive(Clone, Copy, Debug)]
pub struct DecimalFormat {
    /// A single decimal separator, typically `.` or `,`.
    pub separator: char,
    /// Minimum fraction digits. Capped at 32 to bound output size.
    pub minimum_fraction_digits: u8,
}

impl Default for DecimalFormat {
    fn default() -> Self {
        Self {
            separator: '.',
            minimum_fraction_digits: 0,
        }
    }
}

impl NumberFormatter for DecimalFormat {
    fn parse(&self, text: &str) -> Option<f64> {
        if self.separator != '.' && text.contains('.') {
            return None;
        }
        text.replace(self.separator, ".").parse().ok()
    }

    fn format(&self, value: f64) -> String {
        if value.is_nan() {
            return String::new();
        }
        let rounded = display_round(value);
        let mut text = if rounded == 0. {
            "0".to_owned()
        } else {
            rounded.to_string()
        };
        if rounded.is_finite() {
            let digits = usize::from(self.minimum_fraction_digits.min(32));
            if digits > 0 {
                let existing = text.split_once('.').map_or(0, |(_, tail)| tail.len());
                if existing == 0 {
                    text.push('.');
                }
                text.extend(std::iter::repeat_n('0', digits.saturating_sub(existing)));
            }
        }
        text.replace('.', &self.separator.to_string())
    }
}

fn display_round(value: f64) -> f64 {
    if !value.is_finite() || value == 0. {
        return value;
    }
    // Ten significant decimal digits, without mutating the underlying Value.
    format!("{value:.9e}")
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .unwrap_or(value)
}

/// Parameters shared by text commits, programmatic changes and step actions.
#[derive(Clone, Copy, Debug)]
pub struct NumberConfig {
    /// Lower bound; defaults to -f64::MAX.
    pub minimum: f64,
    /// Upper bound; defaults to f64::MAX.
    pub maximum: f64,
    /// Delta for arrow keys and spin buttons.
    pub small_change: f64,
    /// Delta for PageUp / PageDown.
    pub large_change: f64,
    /// Permit out-of-range values and retain invalid editing text.
    pub validation_disabled: bool,
    /// Wrap an overshoot to the opposite bound.
    pub wrap: bool,
    /// Allow arithmetic expressions in editing text.
    pub accepts_expression: bool,
}

impl Default for NumberConfig {
    fn default() -> Self {
        Self {
            minimum: -f64::MAX,
            maximum: f64::MAX,
            small_change: 1.,
            large_change: 10.,
            validation_disabled: false,
            wrap: false,
            accepts_expression: false,
        }
    }
}

/// Cause of a state transition; boundary changes follow the changed endpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberAction {
    Refresh,
    Commit,
    Restore,
    MinimumChanged,
    MaximumChanged,
    SettingsChanged,
    Increase,
    Decrease,
    PageIncrease,
    PageDecrease,
}

/// Result of a transition. Value is retained at full f64 precision.
#[derive(Debug)]
pub struct NumberOutput {
    /// Current numeric value; NaN means empty.
    pub value: f64,
    /// Editing/display text after the transition.
    pub text: String,
    /// Coerced lower bound.
    pub minimum: f64,
    /// Coerced upper bound.
    pub maximum: f64,
    /// Whether the up spin action is enabled.
    pub up_enabled: bool,
    /// Whether the down spin action is enabled.
    pub down_enabled: bool,
    /// False only when a commit encountered syntactically invalid text.
    pub valid_input: bool,
}

/// NaN-to-NaN updates do not produce ValueChanged.
pub fn same_value(left: f64, right: f64) -> bool {
    left == right || (left.is_nan() && right.is_nan())
}

/// Canonical round-trip transport through UI frameworks that only expose f32.
pub fn encode(value: f64) -> String {
    if value.is_nan() {
        String::new()
    } else if value == 0. {
        "0".to_owned()
    } else {
        value.to_string()
    }
}

/// Decodes a canonical value. Empty denotes NaN; malformed values are errors.
pub fn decode(text: &str) -> Result<f64, std::num::ParseFloatError> {
    if text.is_empty() {
        Ok(f64::NAN)
    } else {
        text.parse()
    }
}

/// Applies a NumberBox action using a caller-supplied parser/formatter.
pub fn transition(
    value: f64,
    text: &str,
    mut config: NumberConfig,
    action: NumberAction,
    formatter: &impl NumberFormatter,
) -> NumberOutput {
    if config.minimum > config.maximum {
        if action == NumberAction::MaximumChanged {
            config.minimum = config.maximum;
        } else {
            config.maximum = config.minimum;
        }
    }
    let coerce = |v: f64| {
        if config.validation_disabled || v.is_nan() || (v >= config.minimum && v <= config.maximum)
        {
            v
        } else if v > config.maximum {
            config.maximum
        } else {
            config.minimum
        }
    };
    let mut current = coerce(value);
    let mut output = if same_value(current, value) {
        text.to_owned()
    } else {
        formatter.format(current)
    };
    let mut valid = true;
    let step = match action {
        NumberAction::Increase => Some(config.small_change),
        NumberAction::Decrease => Some(-config.small_change),
        NumberAction::PageIncrease => Some(config.large_change),
        NumberAction::PageDecrease => Some(-config.large_change),
        _ => None,
    };
    if action == NumberAction::Commit || step.is_some() {
        let text = text.trim();
        let parsed = if text.is_empty() {
            Some(f64::NAN)
        } else if config.accepts_expression {
            expression(text, formatter)
        } else {
            formatter.parse(text)
        };
        if let Some(parsed) = parsed {
            current = coerce(parsed);
            output = formatter.format(current);
        } else {
            valid = false;
            if !config.validation_disabled {
                output = formatter.format(current);
            }
        }
    } else if matches!(action, NumberAction::Refresh | NumberAction::Restore) {
        output = formatter.format(current);
    }
    if let Some(delta) = step
        && !current.is_nan()
    {
        let mut next = current + delta;
        if config.wrap {
            if next > config.maximum {
                next = config.minimum;
            } else if next < config.minimum {
                next = config.maximum;
            }
        }
        next = coerce(next);
        if !same_value(next, current) {
            current = next;
            output = formatter.format(current);
        }
    }
    NumberOutput {
        value: current,
        text: output,
        minimum: config.minimum,
        maximum: config.maximum,
        up_enabled: !current.is_nan()
            && (config.wrap || config.validation_disabled || current < config.maximum),
        down_enabled: !current.is_nan()
            && (config.wrap || config.validation_disabled || current > config.minimum),
        valid_input: valid,
    }
}

#[derive(Clone, Copy)]
enum Token {
    Number(f64),
    Operator(char),
    Open,
    Close,
}

fn precedence(op: char) -> u8 {
    match op {
        '^' => 2,
        '*' | '/' => 1,
        _ => 0,
    }
}

/// Evaluates the official operator set, signed numeric literals and left associativity.
/// Syntax errors return None; division by zero returns Some(NaN).
pub fn expression(text: &str, formatter: &impl NumberFormatter) -> Option<f64> {
    let mut tokens = Vec::new();
    let mut offset = 0;
    let mut number_expected = true;
    while offset < text.len() {
        let ch = text[offset..].chars().next()?;
        if ch == ' ' {
            offset += 1;
            continue;
        }
        if number_expected && ch == '(' {
            tokens.push(Token::Open);
            offset += 1;
        } else if !number_expected && ch == ')' {
            tokens.push(Token::Close);
            offset += 1;
        } else if !number_expected && "+-*/^".contains(ch) {
            tokens.push(Token::Operator(ch));
            number_expected = true;
            offset += 1;
        } else if number_expected {
            let start = offset;
            if ch == '-' {
                offset += 1;
            }
            let digits = offset;
            while offset < text.len() {
                let next = text[offset..].chars().next()?;
                if next.is_whitespace() || "+-*/^()".contains(next) {
                    break;
                }
                offset += next.len_utf8();
            }
            if offset == digits {
                return None;
            }
            tokens.push(Token::Number(formatter.parse(&text[start..offset])?));
            number_expected = false;
        } else {
            return None;
        }
        if tokens.len() > 1024 {
            return None;
        }
    }
    if number_expected {
        return None;
    }
    let mut postfix = Vec::new();
    let mut operators = Vec::new();
    for token in tokens {
        match token {
            Token::Number(_) => postfix.push(token),
            Token::Open => operators.push(token),
            Token::Close => {
                while matches!(operators.last(), Some(Token::Operator(_))) {
                    postfix.push(operators.pop()?);
                }
                if !matches!(operators.pop(), Some(Token::Open)) {
                    return None;
                }
            }
            Token::Operator(op) => {
                while matches!(operators.last(), Some(Token::Operator(previous)) if precedence(*previous) >= precedence(op))
                {
                    postfix.push(operators.pop()?);
                }
                operators.push(token);
            }
        }
    }
    while let Some(token) = operators.pop() {
        if matches!(token, Token::Open) {
            return None;
        }
        postfix.push(token);
    }
    let mut values: Vec<f64> = Vec::new();
    for token in postfix {
        match token {
            Token::Number(value) => values.push(value),
            Token::Operator(op) => {
                let right = values.pop()?;
                let left = values.pop()?;
                values.push(match op {
                    '+' => left + right,
                    '-' => left - right,
                    '*' => left * right,
                    '/' => {
                        if right == 0. {
                            return Some(f64::NAN);
                        } else {
                            left / right
                        }
                    }
                    '^' => left.powf(right),
                    _ => return None,
                });
            }
            _ => return None,
        }
    }
    (values.len() == 1).then(|| values[0])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn official_expression_precedence_and_errors() {
        let format = DecimalFormat::default();
        for (input, result) in [
            ("1 + 2^2", 5.),
            ("2^3^2", 64.),
            ("-2^2", 4.),
            ("(2+3)*4", 20.),
            ("1--2", 3.),
            ("3/-2", -1.5),
        ] {
            assert_eq!(expression(input, &format), Some(result), "{input}");
        }
        for invalid in [
            "", "-(2+3)", "+2", "1+", "(1+2", "1 2", "1e-3", "1\t+2", "2(3)",
        ] {
            assert!(expression(invalid, &format).is_none(), "{invalid}");
        }
        assert!(expression("1/0", &format).is_some_and(f64::is_nan));
    }
    #[test]
    fn display_rounding_does_not_round_value_or_transport() {
        let value = 1.23456789012345;
        let result = transition(
            value,
            "",
            NumberConfig::default(),
            NumberAction::Refresh,
            &DecimalFormat::default(),
        );
        assert_eq!(result.value, value);
        assert_eq!(result.text, "1.23456789");
        assert_eq!(decode(&encode(value)).ok(), Some(value));
        let large = 9_007_199_254_740_991.;
        assert_eq!(decode(&encode(large)).ok(), Some(large));
        assert_eq!(
            DecimalFormat {
                separator: ',',
                minimum_fraction_digits: 2
            }
            .format(12.5),
            "12,50"
        );
    }
    #[test]
    fn invalid_disabled_empty_and_step_validation() {
        let format = DecimalFormat::default();
        let base = NumberConfig {
            minimum: 0.,
            maximum: 10.,
            ..Default::default()
        };
        assert_eq!(
            transition(3., "oops", base, NumberAction::Commit, &format).text,
            "3"
        );
        let relaxed = NumberConfig {
            validation_disabled: true,
            ..base
        };
        assert_eq!(
            transition(3., "oops", relaxed, NumberAction::Commit, &format).text,
            "oops"
        );
        assert_eq!(
            transition(3., "oops", relaxed, NumberAction::Increase, &format).value,
            4.
        );
        assert_eq!(
            transition(3., "9", base, NumberAction::Increase, &format).value,
            10.
        );
        let empty = transition(3., "", base, NumberAction::Increase, &format);
        assert!(empty.value.is_nan());
        assert!(!empty.up_enabled);
        assert_eq!(
            transition(3., "99", relaxed, NumberAction::Commit, &format).value,
            99.
        );
    }
    #[test]
    fn bounds_follow_changed_endpoint_and_wrap_overshoot() {
        let format = DecimalFormat::default();
        let config = NumberConfig {
            minimum: 10.,
            maximum: 5.,
            ..Default::default()
        };
        assert_eq!(
            transition(7., "7", config, NumberAction::MinimumChanged, &format).maximum,
            10.
        );
        assert_eq!(
            transition(7., "7", config, NumberAction::MaximumChanged, &format).minimum,
            5.
        );
        let wrap = NumberConfig {
            minimum: 0.,
            maximum: 10.,
            wrap: true,
            small_change: 3.,
            ..Default::default()
        };
        assert_eq!(
            transition(9., "9", wrap, NumberAction::Increase, &format).value,
            0.
        );
        assert_eq!(
            transition(1., "1", wrap, NumberAction::Decrease, &format).value,
            10.
        );
        assert!(same_value(f64::NAN, f64::NAN));
    }
}

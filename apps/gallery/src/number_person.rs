//! Typed Rust models connected to the Slint control interfaces.

use crate::{
    GalleryWindow, NumberBoxAction, NumberBoxLogic, NumberBoxRequest, NumberBoxResult,
    PersonPictureLogic,
};
use fluent_core::number::{self, DecimalFormat, NumberAction, NumberConfig, NumberFormatter};
use slint::ComponentHandle;

pub(super) fn connect(gallery: &GalleryWindow) {
    let numbers = gallery.global::<NumberBoxLogic>();
    numbers.on_evaluate(evaluate);
    numbers.on_equal(|a, b| match (number::decode(&a), number::decode(&b)) {
        (Ok(a), Ok(b)) => number::same_value(a, b),
        _ => a == b,
    });
    numbers.on_byte_length(|text| i32::try_from(text.len()).unwrap_or(i32::MAX));
    numbers.set_ready(true);
    let person = gallery.global::<PersonPictureLogic>();
    person.on_initials(|name| fluent_core::person::initials(&name).into());
    person.set_ready(true);
}

fn evaluate(request: NumberBoxRequest, action: NumberBoxAction) -> NumberBoxResult {
    let result = (|| {
        let property = |name: &str, text: &str, empty: Option<f64>| -> Result<f64, String> {
            if text.is_empty()
                && let Some(value) = empty
            {
                return Ok(value);
            }
            number::decode(text).map_err(|_| format!("{name} must contain a canonical number."))
        };
        let value = property("Value", &request.value, Some(f64::NAN))?;
        let minimum = property("Minimum", &request.minimum, Some(-f64::MAX))?;
        let maximum = property("Maximum", &request.maximum, Some(f64::MAX))?;
        let small = property("SmallChange", &request.small_change, None)?;
        let large = property("LargeChange", &request.large_change, None)?;
        if [minimum, maximum, small, large]
            .into_iter()
            .any(f64::is_nan)
        {
            return Err("Bounds and step sizes cannot be NaN.".to_owned());
        }
        let mut separators = request.decimal_separator.chars();
        let separator = separators.next().unwrap_or('.');
        if separators.next().is_some() || !matches!(separator, '.' | ',') {
            return Err("Use a single decimal separator: . or ,".to_owned());
        }
        let config = NumberConfig {
            minimum,
            maximum,
            small_change: small,
            large_change: large,
            validation_disabled: request.validation_disabled,
            wrap: request.wrap,
            accepts_expression: request.accepts_expression,
        };
        let action = match action {
            NumberBoxAction::Refresh => NumberAction::Refresh,
            NumberBoxAction::Commit => NumberAction::Commit,
            NumberBoxAction::Restore => NumberAction::Restore,
            NumberBoxAction::MinimumChanged => NumberAction::MinimumChanged,
            NumberBoxAction::MaximumChanged => NumberAction::MaximumChanged,
            NumberBoxAction::SettingsChanged => NumberAction::SettingsChanged,
            NumberBoxAction::Increase => NumberAction::Increase,
            NumberBoxAction::Decrease => NumberAction::Decrease,
            NumberBoxAction::PageIncrease => NumberAction::PageIncrease,
            NumberBoxAction::PageDecrease => NumberAction::PageDecrease,
        };
        let format = DecimalFormat {
            separator,
            minimum_fraction_digits: request.minimum_fraction_digits.clamp(0, 32) as u8,
        };
        let mut output = number::transition(value, &request.text, config, action, &format);
        // Slint defers property notifications: a settings notification can observe a
        // new external Value before Value's own notification. Reconcile its display
        // here without committing stale edit text or losing bound-change direction.
        if request.external_value
            && matches!(
                action,
                NumberAction::SettingsChanged
                    | NumberAction::MinimumChanged
                    | NumberAction::MaximumChanged
            )
        {
            output.text = format.format(output.value);
        }
        Ok(output)
    })();
    match result {
        Ok(output) => NumberBoxResult {
            value: number::encode(output.value).into(),
            text: output.text.into(),
            minimum: number::encode(output.minimum).into(),
            maximum: number::encode(output.maximum).into(),
            up_enabled: output.up_enabled,
            down_enabled: output.down_enabled,
            error: Default::default(),
            accessible_minimum: accessible_number(output.minimum),
            accessible_maximum: accessible_number(output.maximum),
            accessible_step: accessible_number(number::decode(&request.small_change).unwrap_or(0.)),
        },
        Err(error) => NumberBoxResult {
            value: request.value,
            text: request.text,
            minimum: request.minimum,
            maximum: request.maximum,
            error: error.into(),
            ..Default::default()
        },
    }
}

fn accessible_number(value: f64) -> f32 {
    value.clamp(-f64::from(f32::MAX), f64::from(f32::MAX)) as f32
}

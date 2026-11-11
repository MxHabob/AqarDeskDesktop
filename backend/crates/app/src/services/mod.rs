pub mod payments;
pub mod properties;
pub mod settings;
pub mod tenants;
pub mod units;

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

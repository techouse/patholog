mod dedupe;
mod key;

pub(crate) use dedupe::first_unique_entries;
pub(crate) use key::comparison_key;

#[cfg(test)]
#[path = "normalize/tests.rs"]
mod tests;

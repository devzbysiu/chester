#![allow(clippy::module_name_repetitions)]

use configuration::tracing::init_tracing;

mod configuration;
mod data_providers;
mod entities;
mod use_cases;

mod result;
mod startup;
#[cfg(test)]
mod testingtools;

fn main() {
    init_tracing();
}

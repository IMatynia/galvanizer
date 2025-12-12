use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};

use crate::config::{ConfigError, ConfigResult};

fn build_globset(patterns: &[String]) -> Result<GlobSet, globset::Error> {
    let mut builder = GlobSetBuilder::new();
    for p in patterns {
        builder.add(Glob::new(p)?);
    }
    builder.build()
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct RootPreferences {
    inclusions: Option<Vec<String>>,
    exclusions: Option<Vec<String>>,
    max_file_size: Option<Vec<String>>,
    substitute_big_files_with_path: Option<bool>,
}

macro_rules! priority_assign {
    ($primary:expr, $supp:expr, $defaults:expr, $field:ident) => {
        $primary
            .as_mut()
            .and_then(|p| p.$field.take())
            .or_else(|| $supp.as_mut().and_then(|s| s.$field.take()))
            .or_else(|| $defaults.$field.clone())
    };
}

impl RootPreferences {
    /// Takes the primary and supplementary preferences and fills out the final preferences struct with the following priority:
    ///
    /// > Take primary value, if it is `None`, take supplementary value, if that is `None` take the default value.
    ///
    /// The value is not guaranteed to be `Some` by the end!
    pub fn fill_with_priority(
        mut primary: Option<RootPreferences>,
        mut supplementary: Option<RootPreferences>,
    ) -> RootPreferences {
        let defaults = RootPreferences::default();
        RootPreferences {
            inclusions: priority_assign!(primary, supplementary, defaults, inclusions),
            exclusions: priority_assign!(primary, supplementary, defaults, exclusions),
            max_file_size: priority_assign!(primary, supplementary, defaults, max_file_size),
            substitute_big_files_with_path: priority_assign!(
                primary,
                supplementary,
                defaults,
                substitute_big_files_with_path
            ),
        }
    }

    pub fn inclusions_globset(&self) -> ConfigResult<GlobSet> {
        build_globset(
            self.inclusions
                .as_ref()
                .unwrap_or(&vec!["**/*".to_string()]) // Include everything by default
                .as_slice(),
        )
        .map_err(ConfigError::PatternError)
    }

    pub fn exclusions_globset(&self) -> ConfigResult<GlobSet> {
        build_globset(
            self.exclusions
                .as_ref()
                .unwrap_or(&vec![]) // Include everything by default
                .as_slice(),
        )
        .map_err(ConfigError::PatternError)
    }

    pub fn exclusions(&self) -> Option<&Vec<String>> {
        self.exclusions.as_ref()
    }

    pub fn max_file_size(&self) -> Option<&Vec<String>> {
        self.max_file_size.as_ref()
    }

    pub fn substitute_big_files_with_path(&self) -> bool {
        self.substitute_big_files_with_path.unwrap_or(false)
    }

    pub fn inclusions(&self) -> Option<&Vec<String>> {
        self.inclusions.as_ref()
    }
}

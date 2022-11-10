// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{collections::HashMap, convert::TryFrom};

use serde::{Serialize, Deserialize};

/// Represents a list of metrics and their associated `disabled` property from the
/// remote-settings configuration store
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MetricsDisabledConfig {
    metrics_disabled: HashMap<String, bool>
}

impl MetricsDisabledConfig {
    /// Creates a new MetricsDisabledConfig
    pub fn new() -> Self {
        Self {
            metrics_disabled: HashMap::new()
        }
    }
}

impl TryFrom<String> for MetricsDisabledConfig {
    fn try_from(json: String) -> Result<Self, Self::Error> {
        match serde_json::from_str(json.as_str()) {
            Ok(config) => Ok(config),
            Err(e) => Err(crate::ErrorKind::Json(e)),
        }
    }

    type Error = crate::ErrorKind;
}

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! This integration test should model how the RLB is used when embedded in another Rust application
//! (e.g. FOG/Firefox Desktop).
//!
//! We write a single test scenario per file to avoid any state keeping across runs
//! (different files run as different processes).

mod common;

use glean::{ClientInfoMetrics, ConfigurationBuilder, TestGetValue};
use once_cell::sync::Lazy;

/// Some user metrics.
mod metrics {
    use glean::private::*;
    use glean::Lifetime;
    use glean_core::CommonMetricData;
    use once_cell::sync::Lazy;

    #[allow(non_upper_case_globals)]
    pub static counter: Lazy<CounterMetric> = Lazy::new(|| {
        CounterMetric::new(CommonMetricData {
            name: "counter".into(),
            category: "sample".into(),
            send_in_pings: vec!["store1".into()],
            lifetime: Lifetime::Ping,
            disabled: false,
            ..Default::default()
        })
    });
}

mod pings {
    use super::*;
    use glean::private::PingType;
    use once_cell::sync::Lazy;

    #[allow(non_upper_case_globals)]
    pub static store1: Lazy<PingType> = Lazy::new(|| {
        common::PingBuilder::new("store1")
            .with_send_if_empty(true)
            .build()
    });
}

/// Test scenario: A reset before Glean was ever initialized.
///
/// Data is recorded into the pre-init queue, then `test_reset_glean` is the
/// first thing to initialize Glean. The reset must discard the queued
/// recordings rather than let its own `initialize` flush them back in.
/// See <https://bugzilla.mozilla.org/show_bug.cgi?id=2067862>.
#[test]
fn reset_before_initialize_discards_preinit_recordings() {
    common::enable_test_logging();

    let dir = tempfile::tempdir().unwrap();
    let tmpname = dir.path().to_path_buf();

    let cfg = ConfigurationBuilder::new(true, tmpname, "glean-reset-before-init")
        .with_server_endpoint("invalid-test-host")
        .build();

    let client_info = ClientInfoMetrics {
        app_build: "1.0.0".to_string(),
        app_display_version: "1.0.0".to_string(),
        channel: Some("testing".to_string()),
        locale: Some("xx-XX".to_string()),
        os_version: None,
    };

    Lazy::force(&pings::store1);

    // NOT calling `initialize` first: this only reaches the pre-init queue.
    metrics::counter.add(1);

    // The first initialization of this process happens in here.
    glean::test_reset_glean(cfg, client_info, true);

    assert_eq!(
        None,
        metrics::counter.test_get_value(None),
        "recordings from before the reset must not be replayed by the reset's own initialize"
    );

    // The replacement dispatcher is usable.
    metrics::counter.add(2);
    assert_eq!(Some(2), metrics::counter.test_get_value(None));

    glean::shutdown();
}

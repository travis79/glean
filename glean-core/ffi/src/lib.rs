#![allow(clippy::redundant_closure)]

// Currently requried to `extern crate` for cbindgen to pick it up
extern crate ffi_support;

use std::convert::TryFrom;
use std::os::raw::c_char;

use ffi_support::{
    define_handle_map_deleter, define_string_destructor, ConcurrentHandleMap, ExternError, FfiStr,
};
use lazy_static::lazy_static;

use glean_core::{metrics::*, CommonMetricData, Glean, Lifetime};

lazy_static! {
    static ref GLEAN: ConcurrentHandleMap<Glean> = ConcurrentHandleMap::new();
    static ref BOOLEAN_METRICS: ConcurrentHandleMap<BooleanMetric> = ConcurrentHandleMap::new();
    static ref STRING_METRICS: ConcurrentHandleMap<StringMetric> = ConcurrentHandleMap::new();
    static ref COUNTER_METRICS: ConcurrentHandleMap<CounterMetric> = ConcurrentHandleMap::new();
}

type RawStringArray = *const *const c_char;

/// Create a vector of strings from a raw C-like string array
unsafe fn from_raw_string_array(arr: RawStringArray, len: i32) -> Vec<String> {
    if arr.is_null() || len == 0 {
        return vec![];
    }

    // FIXME: We should double check for null pointers and handle that instead of crashing
    let arr_ptrs = std::slice::from_raw_parts(arr, len as usize);
    arr_ptrs
        .iter()
        .map(|&p| FfiStr::from_raw(p).into_string())
        .collect()
}

#[no_mangle]
pub extern "C" fn glean_initialize(data_dir: FfiStr, application_id: FfiStr) -> u64 {
    #[cfg(target_os = "android")]
    {
        let _ = std::panic::catch_unwind(|| {
            android_logger::init_once(
                android_logger::Filter::default().with_min_level(log::Level::Debug),
                Some("libglean_ffi"),
            );
            log::debug!("Android logging should be hooked up!")
        });
    }

    let mut err = ExternError::success();
    GLEAN.insert_with_output(&mut err, || {
        let data_dir = data_dir.into_string();
        let application_id = application_id.into_string();
        let mut glean = Glean::new();
        glean.initialize(&data_dir, &application_id);
        log::info!("Glean.rs initialized");
        glean
    })
}

#[no_mangle]
pub extern "C" fn glean_is_initialized(glean_handle: u64) -> u8 {
    let mut err = ExternError::success();
    GLEAN.call_with_output(&mut err, glean_handle, |glean| glean.is_initialized())
}

#[no_mangle]
pub extern "C" fn glean_is_upload_enabled(glean_handle: u64) -> u8 {
    let mut err = ExternError::success();
    GLEAN.call_with_output(&mut err, glean_handle, |glean| glean.is_upload_enabled())
}

#[no_mangle]
pub extern "C" fn glean_set_upload_enabled(glean_handle: u64, flag: u8) {
    let mut err = ExternError::success();
    GLEAN.call_with_output_mut(&mut err, glean_handle, |glean| {
        glean.set_upload_enabled(flag != 0)
    })
}

#[no_mangle]
pub extern "C" fn glean_send_ping(glean_handle: u64, ping_name: FfiStr) {
    let mut err = ExternError::success();
    GLEAN.call_with_output(&mut err, glean_handle, |glean| {
        // FIXME: Error handling
        let _ = glean.send_ping(ping_name.as_str());
    })
}

#[no_mangle]
pub extern "C" fn glean_new_boolean_metric(
    category: FfiStr,
    name: FfiStr,
    send_in_pings: RawStringArray,
    send_in_pings_len: i32,
    lifetime: i32,
    err: &mut ExternError,
) -> u64 {
    BOOLEAN_METRICS.insert_with_output(err, || {
        let send_in_pings = unsafe { from_raw_string_array(send_in_pings, send_in_pings_len) };
        let lifetime = Lifetime::try_from(lifetime)
            .map_err(|_| {
                log::error!("[Bool] Failed to convert from lifetime value {}", lifetime);
            })
            .unwrap();

        BooleanMetric::new(CommonMetricData {
            name: name.into_string(),
            category: category.into_string(),
            send_in_pings,
            lifetime,
            ..Default::default()
        })
    })
}

#[no_mangle]
pub extern "C" fn glean_new_string_metric(
    category: FfiStr,
    name: FfiStr,
    send_in_pings: RawStringArray,
    send_in_pings_len: i32,
    lifetime: i32,
    err: &mut ExternError,
) -> u64 {
    STRING_METRICS.insert_with_output(err, || {
        let send_in_pings = unsafe { from_raw_string_array(send_in_pings, send_in_pings_len) };
        let lifetime = Lifetime::try_from(lifetime)
            .map_err(|_| {
                log::error!(
                    "[String] Failed to convert from lifetime value {}",
                    lifetime
                );
            })
            .unwrap();

        StringMetric::new(CommonMetricData {
            name: name.into_string(),
            category: category.into_string(),
            send_in_pings,
            lifetime,
            ..Default::default()
        })
    })
}

#[no_mangle]
pub extern "C" fn glean_new_counter_metric(
    category: FfiStr,
    name: FfiStr,
    send_in_pings: RawStringArray,
    send_in_pings_len: i32,
    lifetime: i32,
    err: &mut ExternError,
) -> u64 {
    COUNTER_METRICS.insert_with_output(err, || {
        let send_in_pings = unsafe { from_raw_string_array(send_in_pings, send_in_pings_len) };
        let lifetime = Lifetime::try_from(lifetime)
            .map_err(|_| {
                log::error!(
                    "[Counter] Failed to convert from lifetime value {}",
                    lifetime
                );
            })
            .unwrap();

        CounterMetric::new(CommonMetricData {
            name: name.into_string(),
            category: category.into_string(),
            send_in_pings,
            lifetime,
            ..Default::default()
        })
    })
}

#[no_mangle]
pub extern "C" fn glean_counter_add(
    glean_handle: u64,
    metric_id: u64,
    amount: u64,
    error: &mut ExternError,
) {
    GLEAN.call_with_output(error, glean_handle, |glean| {
        let mut err = ExternError::success();
        COUNTER_METRICS.call_with_output(&mut err, metric_id, |metric| {
            metric.add(glean, amount);
        })
    })
}

#[no_mangle]
pub extern "C" fn glean_counter_test_has_value(
    glean_handle: u64,
    metric_id: u64,
    storage_name: FfiStr,
) -> u8 {
    let mut err = ExternError::success();
    GLEAN.call_with_output(&mut err, glean_handle, |glean| {
        let mut err = ExternError::success();
        COUNTER_METRICS.call_with_output(&mut err, metric_id, |metric| {
            metric
                .test_get_value(glean, storage_name.as_str())
                .is_none()
        })
    })
}

#[no_mangle]
pub extern "C" fn glean_counter_test_get_value(
    glean_handle: u64,
    metric_id: u64,
    storage_name: FfiStr,
) -> u64 {
    let mut err = ExternError::success();
    GLEAN.call_with_output(&mut err, glean_handle, |glean| {
        let mut err = ExternError::success();
        COUNTER_METRICS.call_with_output(&mut err, metric_id, |metric| {
            metric.test_get_value(glean, storage_name.as_str()).unwrap()
        })
    })
}

#[no_mangle]
pub extern "C" fn glean_boolean_set(
    glean_handle: u64,
    metric_id: u64,
    value: u8,
    error: &mut ExternError,
) {
    GLEAN.call_with_output(error, glean_handle, |glean| {
        let mut err = ExternError::success();
        BOOLEAN_METRICS.call_with_output(&mut err, metric_id, |metric| {
            metric.set(glean, value != 0);
        })
    })
}

#[no_mangle]
pub extern "C" fn glean_string_set(
    glean_handle: u64,
    metric_id: u64,
    value: FfiStr,
    error: &mut ExternError,
) {
    GLEAN.call_with_output(error, glean_handle, |glean| {
        let mut err = ExternError::success();
        STRING_METRICS.call_with_output(&mut err, metric_id, |metric| {
            let value = value.into_string();
            metric.set(glean, value);
        })
    })
}

#[no_mangle]
pub extern "C" fn glean_ping_collect(
    glean_handle: u64,
    ping_name: FfiStr,
    error: &mut ExternError,
) -> *mut c_char {
    GLEAN.call_with_output(error, glean_handle, |glean| {
        let ping_maker = glean_core::ping::PingMaker::new();
        let data = ping_maker.collect_string(glean.storage(), ping_name.as_str());
        log::info!("Ping({}): {}", ping_name.as_str(), data);
        data
    })
}

define_handle_map_deleter!(GLEAN, glean_destroy_glean);
define_handle_map_deleter!(BOOLEAN_METRICS, glean_destroy_boolean_metric);
define_handle_map_deleter!(STRING_METRICS, glean_destroy_string_metric);
define_handle_map_deleter!(COUNTER_METRICS, glean_destroy_counter_metric);
define_string_destructor!(glean_str_free);
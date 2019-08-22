# Reserved ping names

The Glean SDK reserves all ping names in `send_in_pings` starting with `glean_`.

This currently includes, but is not limited to:

* `glean_client_info`
* `glean_internal_info`

Additionally, only Glean may specify `all_pings`.  This special value has no effect in the client, but indicates to the backend infrastructure that a metric may appear in any ping.
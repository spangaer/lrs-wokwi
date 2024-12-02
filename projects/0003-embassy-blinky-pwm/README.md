Notes:

This project uses esp_hal 0.22.0, in to hook in to the async rmt driver.

Sadly while the code currently in the project _should_ work, it doesn't

The pulse signal looks to be correct, because the led lights up, but the
transmission never returns. On channel 0 everything blocks on channel 1
the other embassy tasks still execute, but the main task stays stuck.

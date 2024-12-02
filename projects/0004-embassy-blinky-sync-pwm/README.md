# What is this?

In 0003-embassy-blinky-pwm I failed to use the "real async" implementation of 
esp_hal RMT. So instead I used a trick here. The RMT peripheral works async
on it's own. Even when the driver is in blocking mode. It's a two step process:

 1. Up to 48 bits (I think specs say 64, but code seems to disagree) can be
    provided to the transmit function. The transmit function copies these to
    the RMD RAM/register, starts the transmission and returns a transaction
    handle.
 2. At an _undefined_ later time, the handle can be used to do a blocking wait
    for transmission completion.

We know that for a single pixel we need to transmit `3 * 8 = 24` bits. We're 
helped here by the 24 bit RGB signal fitting in the RMT RAM. A PWM period of
1600 ns was selected, so the transmission should be complete after 38.4 μs.
Before the blocking wait we make an embassy timed await, which will yield
control. Once the blocking wait is executed it returns _immediately_ and does
not block the executor from continuing. Which allow to maintain a
multi-tasking control flow.


# Pulse encoding

See inspiration (1). A translation had to be made from esp_idf_hal, to esp_hal,
but other than that conceptually it's the same thing.


# Inspiration

 1. Rust NEOPixel driving:
    https://wokwi.com/projects/416077615439293441

 2. PWM timing info (quoted below)
    https://github.com/japaric/ws2812b/blob/master/firmware/README.md#pwm-interface

 3. esp_hal docs
    https://docs.esp-rs.org/esp-hal/esp-hal/0.22.0/esp32c3/esp_hal/rmt/index.html

 4. official specs
    https://docs.espressif.com/projects/esp-idf/en/v4.3/esp32c3/hw-reference/esp32c3/user-guide-devkitm-1.html



### PWM Interface (Inspiration: 2)

To control the LED ring the LED driver uses its PWM (Pulse Width Modulation)
functionality to generate the control signal. As per the datasheet a `0` value
should have a high time of around 350 ns (nanoseconds), and a `1` value should
have a high time of around 900 ns. The low timer of the PWM signal is not
important as long as the period of the PWM is at least 1250 ns.

To meet these requirements a PWM signal with a frequency of 400 KHz (period =
2500 ns) was selected (\*). At a timer frequency of 8 MHz this resulted in a
prescaler value of 1 where each timer tick equals 125 ns (1 / 8e6). A duty cycle
(compare output) value of 3 generates a high time of 375 which is close to the
high time required for a WS2812B `0` value, and a duty cycle value of 7
generates a high time of 875 ns which is close to the high time required for a
WS2812B `1` value.

(\*) WS2812B LEDs can operate at a frequency of 800 KHz but this value wasn't
achievable with the selected hardware.

To use the PWM functionality to generate the control signal the duty cycle value
would need to change every 2500 ns which is the PWM signal period. This timing
constraint is too tight to have the CPU do the duty cycle update so this task
is delegated to the DMA (Direct Memory Access) peripheral.
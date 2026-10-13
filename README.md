# std-tests

Tests for the Rust standard library on the Wii U (`powerpc-nintendo-cafe`), run on real hardware.

## Requirements

- The `cafe` toolchain (see `rust-toolchain.toml`) [subject of change once upstream]
- [`cargo-wiiu`](https://github.com/rust-wiiu/cargo-wiiu)
- A Wii U with the [wiiload](https://github.com/wiiu-env/wiiload_plugin) and [USBSerialLogger](https://github.com/wiiu-env/USBSerialLogger) plugin, on the same network
- A serial (UART) connection to the console (USB Null-Modem-Cable with FT232 chip)

## Setup

Set the console's IP address and serial port in `.cargo/config.toml`:

```toml
[env]
IP = "192.168.8.155"
SERIAL = "/dev/ttyUSB0"
```

Optionally, `TIMEOUT` sets how many seconds to wait for a test run (default: 120).

## Usage

With the console on the Wii U Menu:

```sh
cargo test              # all suites
cargo test --test fs    # one suite
```

`runner.sh` uploads each test binary, prints the test output from the serial port, and then the console returns to the Wii U Menu. Each binary might take up to 2 minutes to complete. After this time, you can assume something went wrong.

## Notes

- Run `cargo clean` after rebuilding std; cargo does not notice the change otherwise.
- Arguments cannot be passed to the console, so test filters (`cargo test -- name`) are ignored.
- Every test file must start with:

    ```rust
    #![feature(custom_test_frameworks)]
    #![test_runner(std_tests::runner)]
    ```

    Without it, the console stays stuck after the tests and has to be restarted.

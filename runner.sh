#!/usr/bin/env bash
# Cargo runner for powerpc-nintendo-cafe on real hardware: uploads the ELF via
# the wiiload plugin and reads libtest's output from the console's UART.
#
# Arguments cannot be forwarded to the console, so extra arguments (e.g.
# libtest filters) are ignored. The run ends at libtest's `test result:` line;
# std's `process::exit` aborts on Cafe, so that line is the only reliable
# signal. Anything else ends in a timeout.
set -euo pipefail

# Usually set in the [env] section of .cargo/config.toml.
WIIU_IP=${IP:?set IP to the IP address of the Wii U}
WIIU_SERIAL=${SERIAL?set SERIAL to the serial port the console is connected to}
TIMEOUT=${TIMEOUT:-120}

elf=${1:?usage: runner.sh <file.elf> [args...]}
if [[ $# -gt 1 ]]; then
    echo "runner: ignoring arguments: ${*:2}" >&2
fi

tmp=$(mktemp -d)
logger_pgid=
cleanup() {
    if [[ -n $logger_pgid ]]; then
        kill -- "-$logger_pgid" 2>/dev/null || true
    fi
    rm -rf "$tmp"
}
trap cleanup EXIT

# Cargo does not notice when the std of a linked toolchain is rebuilt, so the
# test binary may still contain the old std.
std_rlib=$(ls -t "$(rustc --print sysroot)"/lib/rustlib/powerpc-nintendo-cafe/lib/libstd-*.rlib 2>/dev/null | head -1)
if [[ -n $std_rlib && $std_rlib -nt $elf ]]; then
    echo "runner: warning: $(basename "$elf") is older than the cafe std; run \`cargo clean\` to rebuild it" >&2
fi

rpx="$tmp/$(basename "${elf%.elf}").rpx"
log="$tmp/serial.log"
cargo-wiiu wiiu rpx "$elf" "$rpx" >/dev/null

# The logger must be listening before the upload, or the first lines are lost.
setsid cargo-wiiu wiiu logger "$WIIU_SERIAL" >"$log" 2>/dev/null &
logger_pgid=$!
sleep 1
if ! kill -0 "$logger_pgid" 2>/dev/null; then
    echo "runner: cannot open $WIIU_SERIAL" >&2
    exit 1
fi

if ! cargo-wiiu wiiu upload --ip "$WIIU_IP" "$rpx" >/dev/null 2>&1; then
    echo "runner: upload to $WIIU_IP failed (is the wiiload plugin running?)" >&2
    exit 1
fi

echo "runner: uploaded to $WIIU_IP, waiting for test output..." >&2

# Streams libtest's output as it arrives, without the system's own UART
# messages before it.
start_re='^running [0-9]+ tests?$'
result_re='^test result: (ok|FAILED)\.'
printed=-1
code=
for ((i = 0; i < TIMEOUT * 4; i++)); do
    if ((printed < 0)) && start=$(grep -an -m1 -E "$start_re" "$log"); then
        printed=$((${start%%:*} - 1))
    fi
    if line=$(grep -m1 -aoE "$result_re" "$log"); then
        [[ $line == *ok* ]] && code=0 || code=101
    fi
    if ((printed >= 0)); then
        total=$(wc -l <"$log")
        if ((total > printed)); then
            sed -n "$((printed + 1)),${total}p" "$log" | sed '/^test result:/q'
            printed=$total
        fi
    fi
    [[ -n $code ]] && break
    sleep 0.25
done

if [[ -z $code ]]; then
    if ((printed < 0)); then
        cat "$log"
        echo "runner: the tests never started within ${TIMEOUT}s; the console is probably stuck in the previous app, restart it" >&2
    else
        echo "runner: no test result within ${TIMEOUT}s (crash or hang)" >&2
    fi
    exit 124
fi
exit "$code"

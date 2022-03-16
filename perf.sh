perf record -g -F 999 target/debug/rustinator
perf script -F +pid > /tmp/test.perf
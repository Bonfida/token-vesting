HFUZZ_RUN_ARGS="-t 10 -n 16 -N 1000000" cargo hfuzz run vesting_fuzz

cargo hfuzz run-debug vesting_fuzz hfuzz_workspace/*/*.fuzz
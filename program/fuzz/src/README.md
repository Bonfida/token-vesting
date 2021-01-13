BPF_OUT_DIR="/home/lcchy/repos/bonfida/token-vesting/program/target/deploy" HFUZZ_RUN_ARGS="-t 10 -n 1 -N 1000000" cargo hfuzz run vesting_fuzz

BPF_OUT_DIR="/home/lcchy/repos/bonfida/token-vesting/program/tar cargo hfuzz run-debug vesting_fuzz hfuzz_workspace/*/*.fuzz
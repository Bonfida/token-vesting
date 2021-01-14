CMD used to run the fuzzing (run cargo build-bpf at least once before):
```
BPF_OUT_DIR="/home/lcchy/repos/bonfida/token-vesting/program/target/deploy" HFUZZ_RUN_ARGS="-t 10 -n 32 -N 1000000" cargo hfuzz run vesting_fuzz
```

CMD used to debug the last crash:
```
BPF_OUT_DIR="/home/lcchy/repos/bonfida/token-vesting/program/target/deploy" cargo hfuzz run-debug vesting_fuzz hfuzz_workspace/*/*.fuzz
```

`BPF_OUT_DIR` is there to force ProgramTest to load token_vesting as BPF code (see bug mentioned in /README.md).

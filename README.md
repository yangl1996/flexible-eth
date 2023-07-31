# Optimal High-Safety Flexible Consensus (Confirmation Rule for Ethereum)

To reproduce the experiments with our proof-of-concept implementation of the flexible confirmation rule for Ethereum (derived from our optimal high-safety flexible consensus construction):
1. Setup and run an Ethereum full node (see instructions below, requires ~50 hours of time and ~4TB of NVMe SSD storage)
2. Setup and run the proof-of-concept implementation of the flexible confirmation rule for Ethereum (see instructions below)
3. Inspect output (see instructions below)


## Setup of Ethereum Node

### Lighthouse (Consensus Client)

Setup:
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sudo apt install -y git gcc g++ make cmake pkg-config llvm-dev libclang-dev clang protobuf-compiler
git clone https://github.com/sigp/lighthouse.git
cd lighthouse
git checkout dfcb3363c757671eb19d5f8e519b4b94ac74677a
PROFILE=maxperf make -j8
```

Run:
```
lighthouse bn --network mainnet --datadir /home/ubuntu/lighthouse-data --http --execution-endpoint http://localhost:8551 --execution-jwt /home/ubuntu/reth-data/jwt.hex --reconstruct-historic-states --slots-per-restore-point 256 --historic-state-cache-size 8 --checkpoint-sync-url https://mainnet.checkpoint.sigp.io --genesis-backfill --disable-backfill-rate-limiting
```

### Reth (Execution Client)

Setup:
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sudo apt-get install libclang-dev pkg-config build-essential
git clone https://github.com/paradigmxyz/reth
cd reth
git checkout 77faa04ca6682c033ecfe391a0e5672692ba0adb
RUSTFLAGS="-C target-cpu=native" cargo install --profile maxperf --path bin/reth --bin reth
```

Run:
```
reth node --chain mainnet --datadir /home/ubuntu/reth-data --http --http.api all --authrpc.jwtsecret /home/ubuntu/reth-data/jwt.hex --authrpc.addr 127.0.0.1 --authrpc.port 8551
```

### Foundry (Debugging)

Setup:
```
curl -L https://foundry.paradigm.xyz | bash
source /home/ubuntu/.bashrc
foundryup
```

Run:
```
cast rpc eth_syncing
```

### Utils

Setup:
```
apt-get install ifstat htop tmux iotop vnstat net-tools
```


## Confirmation Rule for Ethereum

```
cd flexible-eth/flexibleeth
./run-example.sh cache.rocksdb 2560
```


## Output

After running the proof-of-concept implementation of the confirmation rule for Ethereum (see `run-example.sh` script), the output is found in `output-example-...`.

## References

- [Database schema](https://github.com/yangl1996/flexible-eth/blob/main/flexibleeth/docs/README.md)

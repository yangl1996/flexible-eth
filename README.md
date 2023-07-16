# How To: Setting up Ethereum clients

## Lighthouse:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sudo apt install -y git gcc g++ make cmake pkg-config llvm-dev libclang-dev clang protobuf-compiler
git clone https://github.com/sigp/lighthouse.git
cd lighthouse
git checkout dfcb3363c757671eb19d5f8e519b4b94ac74677a
PROFILE=maxperf make -j8
lighthouse bn --network mainnet --datadir /home/ubuntu/lighthouse-data --http --execution-endpoint http://localhost:8551 --execution-jwt /home/ubuntu/reth-data/jwt.hex --reconstruct-historic-states --slots-per-restore-point 256 --historic-state-cache-size 8 --checkpoint-sync-url https://mainnet.checkpoint.sigp.io --genesis-backfill --disable-backfill-rate-limiting
```

## Reth:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sudo apt-get install libclang-dev pkg-config build-essential
git clone https://github.com/paradigmxyz/reth
cd reth
git checkout 77faa04ca6682c033ecfe391a0e5672692ba0adb
RUSTFLAGS="-C target-cpu=native" cargo install --profile maxperf --path bin/reth --bin reth
reth node --chain mainnet --datadir /home/ubuntu/reth-data --http --http.api all --authrpc.jwtsecret /home/ubuntu/reth-data/jwt.hex --authrpc.addr 127.0.0.1 --authrpc.port 8551
```

## Foundry:

```
curl -L https://foundry.paradigm.xyz | bash
source /home/ubuntu/.bashrc
foundryup
```

## Utils:

```
apt-get install ifstat htop tmux iotop vnstat net-tools
```



## NOT USED: Prysm:

```
beacon-chain --datadir /home/ubuntu/prysm-data --execution-endpoint /home/ubuntu/geth-data/geth.ipc --slots-per-archive-point 32
```

## NOT USED: Geth:

```
geth --http --http.api eth,net,engine,admin --datadir /home/ubuntu/geth-data
```


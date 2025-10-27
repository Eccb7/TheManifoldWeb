# Quick Start Guide

## System Requirements

Before building The Manifold Web, ensure you have the correct versions:

- **Rust**: 1.76.0 or later
- **Python**: 3.10 or later
- **IPFS**: Latest stable release

## Step 1: Update Rust (if needed)

```bash
# Check current Rust version
rustc --version

# Update Rust toolchain
rustup update

# Verify version is 1.76.0 or later
rustc --version
```

## Step 2: Install IPFS

### Linux
```bash
wget https://dist.ipfs.tech/kubo/v0.24.0/kubo_v0.24.0_linux-amd64.tar.gz
tar -xvzf kubo_v0.24.0_linux-amd64.tar.gz
cd kubo
sudo bash install.sh
ipfs --version
```

### Initialize IPFS (first time only)
```bash
ipfs init
```

## Step 3: Build The Manifold Web

```bash
cd /home/ojwang/Desktop/TheManifoldWeb

# Build all Rust crates
cargo build --workspace

# This will take a few minutes on first run
```

## Step 4: Install Python Dependencies

```bash
# Create a virtual environment (recommended)
python3 -m venv venv
source venv/bin/activate

# Install dependencies
pip install -r python/simulation-lab/requirements.txt
```

## Step 5: Run Tests

```bash
# Rust tests (should all pass)
cargo test --workspace

# Python tests
cd python/simulation-lab
pytest -v
cd ../..
```

## Step 6: Start the System

### Terminal 1: IPFS Daemon
```bash
ipfs daemon
```

### Terminal 2: Manifold Node
```bash
cd /home/ojwang/Desktop/TheManifoldWeb
cargo run -p manifold-node
```

### Terminal 3: Observer Client
```bash
cd /home/ojwang/Desktop/TheManifoldWeb
cargo run -p observer-client
```

### Terminal 4: Python Simulation
```bash
cd /home/ojwang/Desktop/TheManifoldWeb/python/simulation-lab
python demo.py
```

## Common Issues

### Issue: Rust version too old

**Error**: `feature 'edition2024' is required`

**Solution**:
```bash
rustup update stable
rustup default stable
```

### Issue: IPFS not running

**Error**: `Failed to connect to IPFS API`

**Solution**:
```bash
# Start IPFS daemon in a separate terminal
ipfs daemon
```

### Issue: Python package missing

**Error**: `ModuleNotFoundError: No module named 'mesa'`

**Solution**:
```bash
pip install -r python/simulation-lab/requirements.txt
```

## What to Expect

### Manifold Node Output
```
🌐 Starting Manifold Node...
📡 Network initialized
🔑 Peer ID: 12D3KooW...
🎧 Listening on /ip4/0.0.0.0/tcp/xxxxx
```

### Observer Client Output
```
👁️  Starting Manifold Observer Client...
📡 Subscribed to topic: manifold-actions
🎧 Listening on /ip4/0.0.0.0/tcp/xxxxx
👁️  Observer running. Monitoring network activity...
```

### Python Simulation Output
```
🌐 The Manifold Web - Agent Simulation
============================================================
Initializing with 10 agents...
Running simulation for 50 steps...
```

## Next Steps

1. **Explore the code**:
   - Start with `crates/manifold-protocol/src/models.rs`
   - Read `docs/ARCHITECTURE.md`

2. **Experiment**:
   - Modify mutation rates in `manifold-node/src/simulation.rs`
   - Add new agent behaviors in Python simulations
   - Create custom genome parameters

3. **Contribute**:
   - See `docs/CONTRIBUTING.md`
   - Report issues on GitHub
   - Submit pull requests

## Documentation

- [README.md](../README.md) - Project overview
- [ARCHITECTURE.md](ARCHITECTURE.md) - System design
- [CONTRIBUTING.md](CONTRIBUTING.md) - How to contribute
- [PROJECT_SETUP.md](PROJECT_SETUP.md) - Complete setup details

## Support

- **GitHub Issues**: https://github.com/Eccb7/TheManifoldWeb/issues
- **Discussions**: https://github.com/Eccb7/TheManifoldWeb/discussions

---

**Ready to explore The Manifold Web!** 🚀

# MetaCog-Bench

Evaluation framework for MetaYGN metacognitive capabilities.

## Setup
pip install -e .

## Running (requires daemon)
# Start daemon first:
cargo run -p metaygn-daemon

# In another terminal, note the port from ~/.claude/aletheia/daemon.port
export METAYGN_PORT=<port>
pytest benchmarks/ -v

# Q-Protocol Telemetry

**A lightweight telemetry and registry protocol for autonomous agent swarms.**

## Overview

The Q-Protocol Telemetry library (`q-protocol-telemetry`) provides a standardized interface for registering autonomous agents and emitting structured telemetry events. It is designed to facilitate the "Q Protocol" standard for inter-agent communication and observability.

## Features

- **Agent Registry**: Centralized (in-memory) or distributed tracking of active agents.
- **Telemetry Emitters**: Standardized event logging for agent state transitions (e.g., `START`, `COMPLETED`, `FAILED`).
- **Signature Verification**: (Placeholder) Mechanisms for verifying agent identity.

## Installation

```bash
pip install q-protocol-telemetry
```

## Usage

```python
from q_protocol_telemetry import SwarmRegistry

registry = SwarmRegistry()
registry.register_agent("agent-alpha", "v1.0.2")

# Emit a heartbeat
registry.emit_telemetry("agent-alpha", "HEARTBEAT", {"load": 0.45})
```

## License

MIT

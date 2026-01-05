# Q-Protocol: Agentic Telemetry Standard

The Universal Z-Order Telemetry Bridge for Autonomous Agents.

[ ![PyPI](https://img.shields.io/pypi/v/q-protocol-telemetry)](https://pypi.org/project/q-protocol-telemetry/) [ ![NPM](https://img.shields.io/npm/v/@philhills/q-protocol-telemetry-js)](https://www.npmjs.com/package/@philhills/q-protocol-telemetry-js)

The **Q-Protocol** is a lightweight, high-density telemetry standard designed for autonomous agent swarms. It utilizes Z-Order curves (Morton Codes) to compress multi-dimensional agent state (location, memory, intent) into a single integer stream, reducing telemetry payload size by up to 40x compared to JSON bloat.

## Structure

*   **`python/`**: The core Python library (`q-protocol-telemetry`) for agent backends.
*   **`javascript/`**: The Node.js/Browser SDK (`q-protocol-telemetry-js`) for frontend visualization and edge nodes.

## Installation

### Python
```bash
pip install q-protocol-telemetry
```

### JavaScript / Node
```bash
npm install @philhills/q-protocol-telemetry-js
```

## Learn More
Read the manifesto: [The Q-Protocol: Reducing Agentic Telemetry Costs with Z-Order Curves](https://dev.to/philhills/the-q-protocol)

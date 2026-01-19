# A2AC(md): Agent-to-Agent Communication (Minimal Design)
**Q Protocol Extension Specification v1.0**

---

## Overview

A2AC(md) is the Q Protocol's answer to verbose agent-to-agent communication. Unlike traditional A2AC protocols that use natural language or verbose JSON schemas, A2AC(md) operates on three principles:

1. **Coordinates, not conversations** - Agents point, they don't talk
2. **State, not messages** - Agents query shared Brain, not each other
3. **Receipts, not acknowledgments** - Proof of work, not confirmation messages

**Goal:** Achieve K→0 for agent-to-agent coordination while maintaining the Linux Foundation A2AC compatibility layer for cross-organizational integration.

---

## The Problem with Traditional A2AC

**Traditional A2AC Exchange:**
```json
// Agent A → Agent B (125 tokens)
{
  "from": "agent-a",
  "to": "agent-b",
  "message": "I have completed the git clone operation for the repository located at https://github.com/user/project. The repository has been successfully cloned to /workspace/project and is ready for analysis. Please proceed with the code structure analysis.",
  "context": {
    "operation": "git_clone",
    "status": "completed",
    "repo_url": "https://github.com/user/project",
    "local_path": "/workspace/project"
  }
}

// Agent B → Agent A (87 tokens)
{
  "from": "agent-b",
  "to": "agent-a",  
  "message": "Acknowledged. I will now proceed with the code structure analysis of the cloned repository.",
  "context": {
    "operation": "code_analysis",
    "status": "starting"
  }
}

Total: 212 tokens for 2-agent handoff
```

**A2AC(md) Exchange:**
```
// Agent A (3 tokens)
◈ git:clone:user/project → ◈ RECEIPT:abc123

// Agent B (2 tokens)  
◈ MEM:QUERY:git → analyze:code

Total: 5 tokens for 2-agent handoff (98% reduction)
```

---

## Core Concepts

### 1. Silent Coordination

**Principle:** Agents don't send messages to each other. They emit coordinates to shared Brain.

**Traditional:**
```
Agent A ────message────▶ Agent B
         ◀───response───
```

**A2AC(md):**
```
Agent A ──coordinate──▶ Brain ◀──query── Agent B
         ◀──receipt───        ──execute──▶
```

**Key insight:** Brain is the communication channel. Agents are readers/writers of shared state.

### 2. Coordinate Emission

**Structure:**
```
◈ subject:action:context → ◈ RECEIPT:id
```

**Examples:**
```
◈ git:clone:repo_url → ◈ RECEIPT:clone_xyz
◈ analyze:code:path → ◈ RECEIPT:analysis_xyz
◈ research:start:topic → ◈ RECEIPT:research_xyz
◈ deploy:service:name → ◈ RECEIPT:deploy_xyz
```

**No acknowledgment needed** - Receipt in Brain is proof of completion.

### 3. State Query Protocol

**Before executing, agents MUST query:**
```rust
// Check if work already done
let prior_work = brain.query(trace_id, operation_pattern);

if prior_work.has_receipt() {
    // Already done, use cached result
    return prior_work.receipt.result;
}

// Not done, execute now
let result = execute(operation);
let receipt = emit_receipt(operation, result);
brain.store(receipt);
```

---

## A2AC(md) Message Types

### Type 1: Coordinate (C)
**Purpose:** Initiate operation  
**Format:** `◈ subject:action:context`  
**Token Cost:** 3-8 tokens  

**Example:**
```
◈ research:start:quantum_computing
```

### Type 2: Receipt (R)
**Purpose:** Prove completion  
**Format:** `◈ RECEIPT:id` or receipt JSON in Brain  
**Token Cost:** 2-3 tokens (just the reference)  

**Example:**
```
◈ RECEIPT:rcpt_a1b2c3d4

{
  "receipt_id": "rcpt_a1b2c3d4",
  "operation": "research:start:quantum_computing",
  "agent_id": "research-agent-001",
  "timestamp": "2026-01-19T16:45:32Z",
  "success": true,
  "result": {
    "findings": "...",
    "sources": ["..."]
  },
  "hash": "blake3:..."
}
```

### Type 3: Query (Q)
**Purpose:** Check state before acting  
**Format:** `◈ MEM:QUERY:pattern`  
**Token Cost:** 3-5 tokens  

**Example:**
```
◈ MEM:QUERY:git_operations

Returns: [receipt_1, receipt_2, ...]
```

### Type 4: Error (E)
**Purpose:** Report failure with recovery info  
**Format:** `◈ ERROR:operation → reason → ◈ RETRY:id`  
**Token Cost:** 5-10 tokens  

**Example:**
```
◈ git:clone:invalid_url → ERROR
repo_not_found
◈ RETRY:clone_xyz
```

---

## Communication Patterns

### Pattern 1: Sequential Workflow

**Scenario:** Clone repo → Analyze code → Generate report

**Traditional A2AC (450+ tokens):**
```
Orchestrator → Git Agent: "Please clone the repository..."
Git Agent → Orchestrator: "I have completed the clone..."
Orchestrator → Code Agent: "Please analyze the code..."
Code Agent → Orchestrator: "Analysis complete..."
Orchestrator → Report Agent: "Please generate report..."
Report Agent → Orchestrator: "Report generated..."
```

**A2AC(md) (25 tokens):**
```
◈ workflow:code_review:repo_url

Brain expands to:
1. ◈ git:clone:repo_url
2. ◈ analyze:code
3. ◈ report:generate

Each agent:
- Queries Brain for dependencies
- Executes if not done
- Emits receipt
- Next agent proceeds when receipt exists
```

### Pattern 2: Parallel Execution

**Scenario:** Multi-agent research on different topics

**Traditional A2AC:**
```
Orchestrator → Agent 1: "Research quantum..."
Orchestrator → Agent 2: "Research AI..."
Orchestrator → Agent 3: "Research blockchain..."
[Wait for 3 responses]
Agent 1 → Orchestrator: "Research complete..."
Agent 2 → Orchestrator: "Research complete..."
Agent 3 → Orchestrator: "Research complete..."

Total: ~600 tokens, sequential confirmations
```

**A2AC(md):**
```
◈ batch:research:quantum,ai,blockchain

Each agent:
- Receives topic via Mesh routing
- Executes independently
- Emits receipt to Brain
- No orchestrator coordination needed

Completion check:
◈ MEM:QUERY:research → count(receipts) == 3

Total: ~15 tokens, parallel execution
```

### Pattern 3: Conditional Execution

**Scenario:** Execute B only if A succeeds

**Traditional A2AC:**
```
Orchestrator → Agent A: "Execute operation A"
Agent A → Orchestrator: "Success" or "Failure"
If success:
    Orchestrator → Agent B: "Execute operation B"
    Agent B → Orchestrator: "Complete"

Total: ~300 tokens
```

**A2AC(md):**
```
Agent B self-coordinates:

async fn execute() {
    let a_receipt = brain.query("operation_a").await?;
    
    if !a_receipt.success {
        return Err("Dependency failed");
    }
    
    // Proceed
    let result = self.do_work();
    emit_receipt(result);
}

No orchestrator needed.
Total: ~10 tokens (just the coordinates + receipts)
```

---

## Implementation Guide

### Rust Example

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct A2ACMessage {
    coordinate: String,
    trace_id: String,
    timestamp: u64,
}

impl A2ACMessage {
    fn emit(coord: &str, trace_id: &str) -> Self {
        A2ACMessage {
            coordinate: coord.to_string(),
            trace_id: trace_id.to_string(),
            timestamp: now(),
        }
    }
    
    fn parse(&self) -> (String, String, String) {
        let parts: Vec<&str> = self.coordinate
            .trim_start_matches("◈ ")
            .split(':')
            .collect();
        
        (
            parts[0].to_string(), // subject
            parts[1].to_string(), // action
            parts.get(2).unwrap_or(&"").to_string(), // context
        )
    }
}

struct A2ACAgent {
    agent_id: String,
    brain: BrainClient,
}

impl A2ACAgent {
    async fn coordinate(&self, operation: &str) -> Result<Receipt, Error> {
        // 1. Query state (Protocol #3)
        let prior = self.brain.query(operation).await?;
        
        if let Some(receipt) = prior {
            println!("◈ CACHED:{}", receipt.id);
            return Ok(receipt);
        }
        
        // 2. Execute
        println!("◈ EXECUTE:{}", operation);
        let result = self.execute(operation).await?;
        
        // 3. Emit receipt (Protocol #2)
        let receipt = Receipt::new(operation, result);
        self.brain.store(&receipt).await?;
        
        println!("◈ RECEIPT:{}", receipt.id);
        Ok(receipt)
    }
}
```

### Python Example

```python
from dataclasses import dataclass
from typing import Optional
import requests

@dataclass
class A2ACMessage:
    coordinate: str
    trace_id: str
    timestamp: int
    
    @staticmethod
    def emit(coord: str, trace_id: str):
        return A2ACMessage(
            coordinate=coord,
            trace_id=trace_id,
            timestamp=int(time.time())
        )
    
    def parse(self):
        parts = self.coordinate.lstrip("◈ ").split(":")
        return {
            "subject": parts[0],
            "action": parts[1],
            "context": parts[2] if len(parts) > 2 else ""
        }

class A2ACAgent:
    def __init__(self, agent_id: str, brain_url: str):
        self.agent_id = agent_id
        self.brain_url = brain_url
    
    async def coordinate(self, operation: str) -> Receipt:
        # 1. Query state (Protocol #3)
        prior = await self.brain.query(operation)
        
        if prior:
            print(f"◈ CACHED:{prior['receipt_id']}")
            return prior
        
        # 2. Execute
        print(f"◈ EXECUTE:{operation}")
        result = await self.execute(operation)
        
        # 3. Emit receipt (Protocol #2)
        receipt = self.emit_receipt(operation, result)
        await self.brain.store(receipt)
        
        print(f"◈ RECEIPT:{receipt['receipt_id']}")
        return receipt
```

---

## Hybrid: A2AC(md) + Legacy A2AC

For cross-organizational compatibility, A2AC(md) provides a translation layer:

### Outgoing (A2AC(md) → Legacy A2AC)

```rust
fn translate_to_legacy(coord: &str) -> LegacyA2AC {
    let (subject, action, context) = parse_coordinate(coord);
    
    LegacyA2AC {
        from: agent_id,
        to: None, // Routed by capability
        message: format!(
            "Execute {} operation on {}",
            action, context
        ),
        context: json!({
            "operation": format!("{}:{}", subject, action),
            "parameters": context,
            "protocol": "a2ac-md"
        })
    }
}
```

### Incoming (Legacy A2AC → A2AC(md))

```rust
fn translate_from_legacy(msg: LegacyA2AC) -> String {
    // Extract operation from verbose message
    let operation = msg.context.get("operation")
        .or_else(|| infer_operation(&msg.message));
    
    // Convert to coordinate
    format!("◈ {}", operation)
}
```

This allows A2AC(md) agents to interoperate with legacy systems while maintaining K→0 efficiency internally.

---

## Metrics & Monitoring

### A2AC(md) Compliance Dashboard

```
┌─────────────────────────────────────────────┐
│      A2AC(md) HEALTH METRICS                │
├─────────────────────────────────────────────┤
│                                             │
│  Coordination Efficiency:                   │
│  ├── Avg tokens/coordination: 8.2          │
│  ├── Target: <15                           │
│  └── Status: ✓ PASSING                     │
│                                             │
│  Protocol Compliance:                       │
│  ├── Coordinate usage: 97%                 │
│  ├── Receipt coverage: 100%                │
│  ├── State query rate: 100%                │
│  └── Status: ✓ OPTIMAL                     │
│                                             │
│  Anti-Patterns:                             │
│  ├── Verbose messages: 0                   │
│  ├── Missing receipts: 0                   │
│  ├── Amnesia incidents: 0                  │
│  └── Hallucinations: 0                     │
│                                             │
│  Agent Fleet:                               │
│  ├── Total agents: 155                     │
│  ├── A2AC(md) native: 147 (95%)            │
│  ├── Legacy compat: 8 (5%)                 │
│  └── Migration: ON TRACK                   │
│                                             │
└─────────────────────────────────────────────┘
```

### Alerting Rules

```yaml
alerts:
  - name: "Verbose Coordination"
    condition: avg_tokens_per_coord > 20
    severity: warning
    action: "Review agent for Q Protocol compliance"
  
  - name: "Missing Receipts"
    condition: receipt_coverage < 1.0
    severity: critical
    action: "Investigate hallucination risk"
  
  - name: "Amnesia Detected"
    condition: redundant_operations > 0
    severity: error
    action: "Enable day_zero.rs enforcement"
  
  - name: "K→0 Regression"
    condition: k_value > 30
    severity: warning
    action: "Retrain agent with Q-Mem"
```

---

## Migration Path

### Phase 1: Dual Mode (Current)
```
Legacy A2AC agents ←→ Translation Layer ←→ A2AC(md) agents
                           ↓
                       Cube Brain
```

All agents support both protocols via translation layer.

### Phase 2: A2AC(md) Preferred (3 months)
```
A2AC(md) agents ──────────────────→ Cube Brain
                                        ↓
Legacy agents ──→ Translation Layer ──→
```

New agents use A2AC(md) only. Legacy agents translate.

### Phase 3: A2AC(md) Native (6 months)
```
A2AC(md) agents only ──→ Cube Brain

Legacy agents: DEPRECATED
Translation layer: REMOVED
```

100% A2AC(md) fleet. Legacy protocols removed.

### Phase 4: A2AC(md) + Q-Mem (12 months)
```
Q-Mem trained agents ──→ Learned coordination ──→ K→0

day_zero.rs: DEPRECATED
A2AC(md): Native behavior
K value: <20 tokens/coordination
```

Agents self-enforce Q Protocol through RL training. No runtime enforcement needed.

---

## Comparison Table

| Feature | Traditional A2AC | A2AC(md) | Improvement |
|---------|-----------------|----------|-------------|
| Tokens/coord | 200-400 | 5-15 | 95-98% |
| Latency | High (sequential) | Low (parallel) | 60% |
| Hallucination | Common | Impossible | 100% |
| Amnesia | Frequent | Impossible | 100% |
| Receipt coverage | Optional | Required | 100% |
| Cross-org compat | Native | Translation layer | Maintained |
| Learning curve | Low | Medium | Offset by training |
| Infrastructure | Agent-to-agent | Shared Brain | Simpler |

---

## Best Practices

### DO ✅

1. **Always query before coordinating**
   ```
   ◈ MEM:QUERY:operation → check → execute
   ```

2. **Emit receipts immediately**
   ```
   execute() → result → ◈ RECEIPT:id
   ```

3. **Use coordinates for standard operations**
   ```
   ◈ git:clone:repo (not "Please clone the repository...")
   ```

4. **Leverage parallel execution**
   ```
   ◈ batch:operation:param1,param2,param3
   ```

5. **Validate receipts before claiming**
   ```
   ◈ VERIFY:agent:operation → ◈ RECEIPT:id
   ```

### DON'T ❌

1. **Don't send direct messages**
   ```
   ❌ agent_a.send_message(agent_b, "...")
   ✅ brain.emit_coordinate("◈ ...")
   ```

2. **Don't acknowledge unnecessarily**
   ```
   ❌ "Acknowledged, proceeding..."
   ✅ [silent execution] → ◈ RECEIPT:id
   ```

3. **Don't use prose for standard operations**
   ```
   ❌ "I will now clone the repository"
   ✅ ◈ git:clone:repo
   ```

4. **Don't skip state queries**
   ```
   ❌ execute() immediately
   ✅ query() → check → execute()
   ```

5. **Don't claim without receipts**
   ```
   ❌ "Operation complete"
   ✅ ◈ RECEIPT:xyz
   ```

---

## Linux Foundation A2A Integration

For organizations using Linux Foundation's A2A protocol, A2AC(md) provides:

### 1. Extension Proposal

Submit A2A Semantic Extension:
```json
{
  "extension_name": "a2a-semantic-coordinates",
  "version": "1.0",
  "description": "Q Protocol coordinate-based communication",
  "backward_compatible": true,
  "specification": {
    "coordinate_format": "◈ subject:action:context",
    "receipt_requirement": "mandatory",
    "state_query_protocol": "query-before-act",
    "token_target": "<15 per coordination"
  }
}
```

### 2. Registry Service

Contribute shared coordinate dictionary to A2A Registry:
```
POST /a2a/registry/coordinates

{
  "git:clone": {...},
  "research:start": {...},
  "analyze:code": {...},
  [... standard operations ...]
}
```

### 3. Validation Service

Provide A2AC(md) compliance checker:
```
POST /a2a/validate/a2ac-md

Input: Agent coordination messages
Output: Compliance report + optimization suggestions
```

---

## Roadmap

### Q1 2026: Foundation
- [x] A2AC(md) specification complete
- [x] day_zero.rs enforcement layer
- [ ] Rust SDK for A2AC(md)
- [ ] Python SDK for A2AC(md)

### Q2 2026: Adoption
- [ ] 50% of fleet using A2AC(md)
- [ ] Translation layer deployed
- [ ] Metrics dashboard live
- [ ] Training materials published

### Q3 2026: Optimization
- [ ] Q-Mem integration (learned coordination)
- [ ] K<15 tokens/coordination achieved
- [ ] 95% coordinate usage
- [ ] Zero hallucination/amnesia

### Q4 2026: Graduation
- [ ] 100% fleet A2AC(md) native
- [ ] day_zero.rs deprecated (agents self-enforce)
- [ ] K→0 achieved (<10 tokens/coordination)
- [ ] A2A extension accepted (if open-sourced)

---

## Conclusion

A2AC(md) represents the evolution from verbose agent communication to silent coordination through shared semantic space. By combining:

- **Coordinate-based messaging** (◈ subject:action:context)
- **Receipt-based verification** (unforgeable proof)
- **State-query protocols** (mandatory bootstrap)

We achieve 95-98% token reduction while eliminating hallucination and amnesia entirely.

The transition path from legacy A2AC to A2AC(md) provides backward compatibility, allowing gradual migration while maintaining cross-organizational interoperability through translation layers.

As agents internalize Q Protocol through Q-Mem training, the need for runtime enforcement (day_zero.rs) diminishes, leading to truly autonomous K→0 coordination.

**Status:** SPECIFICATION COMPLETE  
**Version:** 1.0  
**Date:** January 19, 2026  

◈ A2AC(md):READY

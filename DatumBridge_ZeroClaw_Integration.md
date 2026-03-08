# DatumBridge ↔ DTBClaw Integration

## Master–Slave Model

| Role | Component | Responsibility |
|------|-----------|----------------|
| **Master** | DatumBridge | Orchestrator, policy controller, permission gating. Defines *what* DTBClaw may do. |
| **Slave** | DTBClaw | Autonomous agent with its own brain (LLM, agent loop, tools). Must operate *within* master's constraints. |

**Key principle:** DTBClaw retains its brain and can run its own agent loop, but it **must adhere to restrictive conditions and permissions** from DatumBridge (master).

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  DatumBridge (MASTER)                                                        │
│                                                                              │
│  • Workflow orchestration (LangGraph, stages, routing)                        │
│  • Policy engine: allowed tools, guardrails, rate limits, scopes              │
│  • Permission injection: sends constraints when delegating to DTBClaw         │
│  • Audit: receives execution reports, can approve/reject proposed actions    │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ WebSocket + Policy Payload
                                    │ e.g. {"type":"message","content":"...","policy":{...}}
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  DTBClaw (SLAVE)                                                            │
│                                                                              │
│  • Own brain: LLM, agent loop, tools, memory, skills                         │
│  • Receives task + policy constraints from master                           │
│  • Enforces: only allowed tools, no forbidden actions, guardrails           │
│  • Reports back: tool calls, results, compliance status                     │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Policy / Permission Model

DatumBridge injects policy constraints that DTBClaw must enforce before and during execution.

### Example Policy Payload (Master → Slave)

```json
{
  "type": "message",
  "content": "Analyze the logs in /var/log and summarize errors",
  "session_id": "workflow-123-stage-2",
  "policy": {
    "allowed_tools": ["file_read", "memory"],
    "forbidden_tools": ["shell", "file_write"],
    "max_tool_calls": 10,
    "max_tokens": 4000,
    "allowed_paths": ["/var/log"],
    "forbidden_paths": ["/etc/passwd", "/root"],
    "timeout_secs": 60,
    "require_approval": ["shell", "file_write"]
  }
}
```

### DTBClaw Enforcement Behavior

| Constraint | DTBClaw behavior |
|------------|-------------------|
| `allowed_tools` | Only these tools are available in the agent loop |
| `forbidden_tools` | These tools are hidden or return "permission denied" |
| `allowed_paths` | File tools only operate under these paths |
| `forbidden_paths` | Block access to these paths |
| `max_tool_calls` | Stop after N tool invocations |
| `require_approval` | Pause and send proposed action to master; wait for approval before executing |

---

## Communication Flow

### Option A: WebSocket with Policy in Message

Extend WebSocket protocol so each message can carry policy:

```text
Client (DatumBridge) → Server (DTBClaw):
  {"type":"message","content":"<task>","policy":{...},"session_id":"..."}

Server (DTBClaw) → Client (DatumBridge):
  {"type":"chunk","content":"..."}
  {"type":"tool_call","name":"file_read","args":{...}}
  {"type":"approval_required","name":"shell","args":{...}}  // if require_approval
  {"type":"done","full_response":"..."}
```

### Option B: Policy at Connection Time

Establish policy when opening the WebSocket:

```text
ws://dtbclaw:8080/ws/chat?session_id=...&policy=<base64-encoded-policy>
```

Or via initial handshake message:

```text
{"type":"policy","policy":{...}}
{"type":"message","content":"<task>"}
```

### Option C: Policy from Master API

DatumBridge stores policy per workflow/session. ZeroClaw fetches policy from DatumBridge API before running:

```text
DTBClaw: GET https://datumbridge/api/v1/policy?session_id=...
DatumBridge: returns policy JSON
DTBClaw: applies policy, then runs agent loop
```

---

## Implementation Considerations

### DTBClaw (Slave) Changes

1. **Policy parser**: Accept policy from WebSocket message, query param, or API.
2. **Tool filter**: Before agent loop, filter `all_tools()` by `allowed_tools` / `forbidden_tools`.
3. **Path guard**: In file/shell tools, check `allowed_paths` / `forbidden_paths` before execution.
4. **Approval gate**: For `require_approval` tools, emit `approval_required` and block until master sends `approve` / `reject`.
5. **Limits**: Enforce `max_tool_calls`, `max_tokens`, `timeout_secs`.

### DatumBridge (Master) Changes

1. **Policy engine**: Define per-workflow stage policies (allowed tools, paths, etc.).
2. **WebSocket client**: Connect to DTBClaw, send `message` + `policy`. ✅ Implemented (`internal/dtbclaw/client.go`).
3. **Approval handler**: When DTBClaw sends `approval_required`, present to human or auto-approve; send `approve` / `reject`.
4. **Registry**: Map workflow stages to DTBClaw instances + policy bundles. ✅ `dtbclaw_ws` node type + MCBP `handlerType: "dtbclaw_ws"`.

#### Option A Implementation Status (DatumBridge)

- **`dtbclaw_ws` node type**: Added to graph and adapter types; MCBP converter maps `handlerType: "dtbclaw_ws"` → `NodeTypeDTBClawWS`.
- **WebSocket client**: `internal/dtbclaw/client.go` — connects to `ws://{DTBCLAW_WS_URL}/ws/chat`, sends `{"type":"message","content":"...","policy":{...},"session_id":"..."}`, waits for `{"type":"done","full_response":"..."}`.
- **Executor**: `executeDTBClawNode` — reads `message` from node parameters or state `task`, builds policy from node `parameters`/`metadata`, calls client.
- **Config**: `DTBCLAW_WS_URL` (required for dtbclaw_ws nodes), `DTBCLAW_TOKEN` (optional pairing token).

#### Using `dtbclaw_ws` in a workflow

- **Message source**: Node `parameters.message` or state `task` (from upstream).
- **Policy source**: Node `parameters` or `metadata` with keys: `allowed_tools`, `forbidden_tools`, `allowed_paths`, `forbidden_paths`, `max_tool_calls`, `max_tokens`, `timeout_secs`, `require_approval`.
- **MCBP stage**: Set `metadata.handlerType: "dtbclaw_ws"`; optionally add policy fields in `metadata`.

---

## Summary

- **DatumBridge = Master**: Defines *what* DTBClaw may do via policy and permissions.
- **DTBClaw = Slave**: Keeps its brain and agent loop, but *enforces* master's constraints.
- **Integration**: WebSocket (`/ws/chat`) with policy payload; DTBClaw filters tools and actions based on policy; optional approval flow for sensitive actions.

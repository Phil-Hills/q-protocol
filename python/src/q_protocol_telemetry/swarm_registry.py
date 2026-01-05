import time
import json
import logging
from typing import Dict, Any, Optional
from datetime import datetime, timezone

logger = logging.getLogger("q_protocol")

class SwarmRegistry:
    """
    A registry for tracking autonomous agents and their telemetry states
    compliant with Q Protocol v1.0.
    """
    
    def __init__(self):
        self._agents: Dict[str, Dict[str, Any]] = {}
        self._telemetry_log: list = []

    def register_agent(self, agent_id: str, version: str, capabilities: list = None) -> bool:
        """
        Registers a new agent in the swarm.
        
        Args:
            agent_id: Unique identifier for the agent.
            version: Semantic version of the agent code.
            capabilities: List of capability strings.
            
        Returns:
            True if registration was successful.
        """
        if capabilities is None:
            capabilities = []
            
        timestamp = datetime.now(timezone.utc).isoformat()
        
        self._agents[agent_id] = {
            "version": version,
            "capabilities": capabilities,
            "registered_at": timestamp,
            "last_heartbeat": timestamp,
            "status": "ACTIVE"
        }
        
        logger.info(f"Agent Registered: {agent_id} v{version}")
        return True

    def emit_telemetry(self, agent_id: str, event_type: str, payload: Dict[str, Any] = None) -> str:
        """
        Emits a telemetry event for a registered agent.
        
        Args:
            agent_id: The agent emitting the event.
            event_type: strict UPPERCASE event type (e.g., 'TASK_START').
            payload: Arbitrary JSON-serializable data.
            
        Returns:
            The event ID generated for this telemetry.
        """
        if agent_id not in self._agents:
            raise KeyError(f"Agent {agent_id} not registered.")
            
        if payload is None:
            payload = {}
            
        timestamp = datetime.now(timezone.utc).isoformat()
        event_id = f"evt_{int(time.time()*1000)}"
        
        event_record = {
            "event_id": event_id,
            "timestamp": timestamp,
            "agent_id": agent_id,
            "type": event_type.upper(),
            "payload": payload
        }
        
        self._telemetry_log.append(event_record)
        
        # Update heartbeat if applicable
        self._agents[agent_id]["last_heartbeat"] = timestamp
        
        logger.debug(f"Telemetry [{agent_id}]: {event_type} - {json.dumps(payload)}")
        
        return event_id

    def get_agent_status(self, agent_id: str) -> Optional[Dict[str, Any]]:
        """Retrieve the current status of an agent."""
        return self._agents.get(agent_id)

    def dump_telemetry(self) -> str:
        """Exports all telemetry as a JSON string."""
        return json.dumps(self._telemetry_log, indent=2)

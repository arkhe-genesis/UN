#!/usr/bin/env python3
# -*- coding: utf-8 -*-

from __future__ import annotations
from typing import Any, Dict

from agent.tools.base import BaseTool, ToolResult

class RecorderTool(BaseTool):
    name = "recorder"
    description = "Obtém estatísticas recentes ou histórico do HybridRecorder."

    def __init__(self, memory: Any):
        super().__init__()
        self._memory = memory

    async def run(self, **kwargs) -> ToolResult:
        limit = kwargs.get("limit", 10)
        history = await self._memory.get_full_history(limit)
        return ToolResult(success=True, data=history)

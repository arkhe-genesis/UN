#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Cathedral ARKHE — Main Agent Loop
Selo: CATHEDRAL-ARKHE-AGENT-LOOP-2026-06-16

Integra:
- ReActPlanner (raciocínio + ação)
- HybridMemory (curto prazo + RAG + persistência)
- CathedralBridge (comunicação com o core Rust)
- Ferramentas (PicoAds, CoreTick, etc.)
- Guardrails (segurança ética)

Fornece um loop assíncrono completo para agentes autônomos,
com ancoragem opcional na TemporalChain.
"""

import asyncio
import logging
import sys
from typing import Dict, Any, Optional, List
from pathlib import Path

# Importações internas (assumindo estrutura de pacotes)
from agent.planning.react_planner import ReActPlanner, ReActStep
from agent.memory.hybrid_memory import HybridMemory
from agent.tools.base import BaseTool, ToolResult
from agent.guardrails.safety import SafetyGuardrails, SafetyVerdict
from integration.cathedral_bridge import CathedralBridge

# Tentativa de importar o LLM engine (pode ser stub se não disponível)
try:
    from llm.inference.engine import CathedralLLMEngine
except ImportError:
    # Stub para desenvolvimento
    class CathedralLLMEngine:
        async def chat(self, messages, temperature=0.7, max_tokens=1024):
            return "Thought: I need to fetch recommendations.\nAction: picoads\nAction Input: {\"user_context_hash\": \"test\"}"

# Configuração de logging
logging.basicConfig(level=logging.INFO, format='[%(asctime)s] %(levelname)s: %(message)s')
logger = logging.getLogger("cathedral_agent")

class CathedralAgent:
    """
    Agente principal da Cathedral ARKHE.
    Gerencia o ciclo completo: memória → planejamento → execução → segurança.
    """

    def __init__(
        self,
        llm_model_path: Optional[str] = None,
        memory_db_path: str = "cathedral_memory.db",
        use_vector_db: bool = True,
        anchor_to_temporal: bool = False,
        core_mode: str = "auto",          # "auto", "http", "napi"
        core_http_url: str = "http://localhost:8000",
        max_react_iterations: int = 5,
        enable_guardrails: bool = True,
    ):
        """
        Inicializa todos os componentes do agente.

        Args:
            llm_model_path: Caminho para o modelo LLM (se None, usa stub)
            memory_db_path: Caminho do banco SQLite para memória persistente
            use_vector_db: Habilita RAG com ChromaDB
            anchor_to_temporal: Se True, ancora cada passo na TemporalChain
            core_mode: "auto" (tenta napi, fallback HTTP), "http", "napi"
            core_http_url: URL do core quando em modo HTTP
            max_react_iterations: Número máximo de passos ReAct
            enable_guardrails: Ativa verificações de segurança
        """
        # 1. LLM Engine
        if llm_model_path:
            self.llm = CathedralLLMEngine(llm_model_path)
        else:
            logger.warning("No LLM model path provided, using stub engine.")
            self.llm = CathedralLLMEngine()  # stub

        # 2. Memória híbrida
        self.memory = HybridMemory(
            db_path=memory_db_path,
            use_vector_db=use_vector_db,
            anchor_to_temporal=anchor_to_temporal
        )

        # 3. Ponte para o core Rust (EmbodiedCognitiveCore)
        self.bridge = CathedralBridge(mode=core_mode, http_url=core_http_url)

        # 4. Planejador ReAct
        self.planner = ReActPlanner(
            llm_engine=self.llm,
            max_iterations=max_react_iterations,
            anchor_to_temporal=anchor_to_temporal
        )

        # 5. Guardrails de segurança
        self.guardrails = SafetyGuardrails() if enable_guardrails else None

        # 6. Registro de ferramentas
        self.tools: Dict[str, BaseTool] = {}
        self._register_default_tools()

        logger.info("CathedralAgent initialized successfully")

    def _register_default_tools(self):
        """Registra as ferramentas padrão do agente."""
        # Importações locais para evitar dependências circulares
        from agent.tools.picoads_tool import PicoAdsTool
        from agent.tools.core_tick_tool import CoreTickTool   # será criado se necessário
        from agent.tools.recorder_tool import RecorderTool

        # PicoAds Tool
        try:
            self.tools["picoads"] = PicoAdsTool()
        except Exception as e:
            logger.warning(f"Could not load PicoAdsTool: {e}")

        # Core Tick Tool (usa a bridge)
        class DynamicCoreTickTool(BaseTool):
            name = "core_tick"
            description = "Executa um tick no EmbodiedCognitiveCore, atualizando o estado interno (C, I, E)."

            async def run(self, **kwargs) -> ToolResult:
                try:
                    result = await self._bridge.tick()
                    return ToolResult(success=True, data=result)
                except Exception as e:
                    return ToolResult(success=False, error=str(e))

        # Precisamos injetar a bridge na tool – faremos dinamicamente
        if hasattr(self, 'bridge'):
            core_tick = DynamicCoreTickTool()
            core_tick._bridge = self.bridge
            self.tools["core_tick"] = core_tick

        # Recorder Tool (acessa memória)
        class DynamicRecorderTool(BaseTool):
            name = "recorder"
            description = "Obtém estatísticas recentes ou histórico do HybridRecorder."

            async def run(self, **kwargs) -> ToolResult:
                limit = kwargs.get("limit", 10)
                history = await self._memory.get_full_history(limit)
                return ToolResult(success=True, data=history)

        rec_tool = DynamicRecorderTool()
        rec_tool._memory = self.memory
        self.tools["recorder"] = rec_tool

        logger.info(f"Registered tools: {list(self.tools.keys())}")

    async def _check_action_safety(self, tool_name: str, action_input: Any) -> SafetyVerdict:
        """Aplica guardrails antes de executar uma ação."""
        if self.guardrails is None:
            return SafetyVerdict(True, "Guardrails disabled", "log_only")
        return await self.guardrails.check_action(tool_name, action_input)

    async def _check_response_safety(self, response: str) -> SafetyVerdict:
        """Verifica a resposta final antes de entregar ao usuário."""
        if self.guardrails is None:
            return SafetyVerdict(True, "Guardrails disabled", "log_only")
        return await self.guardrails.check_response(response)

    async def _execute_tool(self, action: str, action_input: Any) -> Any:
        """Executa uma ferramenta de forma segura, atualizando a memória com os resultados."""
        safety = await self._check_action_safety(action, action_input)
        if not safety.allowed:
            logger.warning(f"Blocked action {action}: {safety.reason}")
            observation = f"Action blocked by guardrails: {safety.reason}"
        else:
            tool = self.tools.get(action)
            if not tool:
                observation = f"Error: Tool '{action}' not registered."
                logger.error(observation)
            else:
                try:
                    result: ToolResult = await tool(action_input)
                    if result.success:
                        observation = result.data
                    else:
                        observation = f"Tool error: {result.error}"
                except Exception as e:
                    observation = f"Exception during tool execution: {str(e)}"
                    logger.exception(f"Tool {action} crashed")

        # Atualizar memória com o passo completo
        step_entry = {
            "action": action,
            "action_input": action_input,
            "observation": observation
        }
        await self.memory.add(step_entry, observation)

        return observation

    async def run(self, user_input: str) -> Dict[str, Any]:
        """
        Executa o agente em uma consulta do usuário.

        Args:
            user_input: Entrada textual do usuário.

        Returns:
            Dicionário com:
                - "final_answer": resposta final (string)
                - "steps": lista de ReActStep executados
                - "success": booleano
                - "error": mensagem de erro (se houver)
        """
        logger.info(f"Processing user input: {user_input[:100]}...")

        # 1. Recuperar contexto da memória
        context = await self.memory.retrieve(user_input)
        logger.debug(f"Retrieved context length: {len(context)} chars")

        # 2. Planejar usando ReAct (o planner chama o LLM e executa ferramentas no loop)
        try:
            steps: List[ReActStep] = await self.planner.plan(
                user_query=user_input,
                context=context,
                tools={name: tool for name, tool in self.tools.items()},
                executor=self._execute_tool
            )
        except Exception as e:
            logger.exception("Planning failed")
            return {
                "final_answer": f"Planning error: {str(e)}",
                "steps": [],
                "success": False,
                "error": str(e)
            }

        # O planner já executou os passos através do `_execute_tool` com checagens de segurança.

        # Pega a resposta final
        final_answer = None
        for step in steps:
            if step.action == "FINISH":
                final_answer = step.action_input
                break

        # Se o loop terminou sem final_answer (max iterações), pegar o último observation
        if final_answer is None and steps:
            last_step = steps[-1]
            final_answer = last_step.observation if last_step.observation else "Agent could not produce a final answer."

        # Verificar segurança da resposta final
        if final_answer:
            safety = await self._check_response_safety(str(final_answer))
            if not safety.allowed:
                logger.warning(f"Final answer blocked: {safety.reason}")
                final_answer = f"I cannot provide that response due to content restrictions: {safety.reason}"

        logger.info(f"Agent finished. Final answer length: {len(str(final_answer))}")
        return {
            "final_answer": final_answer,
            "steps": steps,
            "success": True,
            "error": None
        }

    async def close(self):
        """Limpeza dos recursos (fechar conexões HTTP, etc.)."""
        await self.bridge.close()
        # Não fechamos memória ou LLM explicitamente

    # --- CLI entry point ---
    @classmethod
    def cli(cls):
        """Executa o agente em modo interativo via linha de comando."""
        import asyncio
        import argparse

        parser = argparse.ArgumentParser(description="Cathedral ARKHE Agent CLI")
        parser.add_argument("--query", type=str, help="Consulta única (modo não interativo)")
        parser.add_argument("--model", type=str, help="Caminho do modelo LLM (opcional)")
        parser.add_argument("--memory-db", type=str, default="cathedral_memory.db")
        parser.add_argument("--core-url", type=str, default="http://localhost:8000")
        args = parser.parse_args()

        agent = cls(
            llm_model_path=args.model,
            memory_db_path=args.memory_db,
            use_vector_db=False,   # CLI pode ser leve
            anchor_to_temporal=False,
            core_http_url=args.core_url
        )

        async def run_interactive():
            if args.query:
                result = await agent.run(args.query)
                print("\n=== AGENT RESPONSE ===")
                print(result["final_answer"])
                print("\n=== STEPS ===")
                for step in result["steps"]:
                    print(f"[{step.step_index}] {step.action}: {step.observation}")
            else:
                print("Cathedral Agent Interactive Mode. Type 'exit' to quit.")
                while True:
                    try:
                        user_input = input("\n> ")
                        if user_input.lower() in ("exit", "quit"):
                            break
                        result = await agent.run(user_input)
                        print("\n🤖:", result["final_answer"])
                    except KeyboardInterrupt:
                        break
                    except Exception as e:
                        print(f"Error: {e}")

        try:
            asyncio.run(run_interactive())
        finally:
            asyncio.run(agent.close())

if __name__ == "__main__":
    CathedralAgent.cli()

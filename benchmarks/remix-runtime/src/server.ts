import { Hono } from 'hono';
import { cors } from 'hono/cors';
import { logger } from 'hono/logger';
import { Compiler } from '@remix-project/remix-solidity';
import { EthDebugger } from '@remix-project/remix-debug';
import { PluginManager, Engine, Plugin } from '@remix-project/engine';
import * as crypto from 'crypto';

const app = new Hono();

app.use('*', cors());
app.use('*', logger());

// Estado global
const state = {
  compiler: null as any,
  debugSessions: new Map<string, any>(),
  pluginManager: null as PluginManager | null,
  engine: null as Engine | null,
};

// Inicialização
async function init() {
  // Compiler
  state.compiler = new Compiler((fileurl: string, cb: Function) => {
    // handleImportCall
    cb(null, '');
  });

  // Plugin Engine
  state.pluginManager = new PluginManager();
  state.engine = new Engine();
  await state.engine.onload();

  // Registra plugins
  const consolePlugin = new Plugin({
    name: 'console',
    methods: ['log', 'error', 'warn']
  });
  state.engine.register([state.pluginManager, consolePlugin]);
  state.pluginManager.activatePlugin('console');
}

// Rota: Compilação
app.post('/api/compile', async (c) => {
  const body = await c.req.json();
  const { source, version, optimize, runs } = body;

  return new Promise((resolve) => {
    state.compiler!.set('version', version);
    state.compiler!.set('optimize', optimize);
    state.compiler!.set('runs', runs);

    state.compiler!.compile({ 'contract.sol': source }, 'contract.sol');

    // Event listener para resultado
    state.compiler!.event.on('compilationFinished', (data: any) => {
      if (data.errors && data.errors.length > 0) {
        resolve(c.json({
          success: false,
          error: data.errors[0].message
        }));
      } else {
        const contract = data.contracts['contract.sol'];
        const contractName = Object.keys(contract)[0];
        const abi = contract[contractName].abi;
        const bytecode = contract[contractName].evm.bytecode.object;
        const bytecodeHash = crypto
          .createHash('sha256')
          .update(bytecode, 'hex')
          .digest('hex');

        resolve(c.json({
          success: true,
          abi,
          bytecode,
          bytecode_hash: bytecodeHash,
          ast: data.sources['contract.sol'].ast
        }));
      }
    });
  });
});

// Rota: Debug Session
app.post('/api/debug/session', async (c) => {
  const { tx_hash, network } = await c.req.json();
  const sessionId = crypto.randomUUID();

  const debuggerInstance = new EthDebugger({
    compilationResult: () => ({})
  });

  // Configura provider (simplificado)
  // debuggerInstance.addProvider(web3, network);

  state.debugSessions.set(sessionId, {
    debugger: debuggerInstance,
    currentStep: 0,
    tx_hash,
  });

  return c.text(sessionId);
});

// Rota: Debug Step
app.post('/api/debug/step', async (c) => {
  const { session_id, step } = await c.req.json();
  const session = state.debugSessions.get(session_id);

  if (!session) {
    return c.json({ error: 'Session not found' }, 404);
  }

  // Executa passo (simplificado)
  // const state = await session.debugger.step(step);

  return c.json({
    session_id,
    current_step: step,
    call_stack: [],
    locals: {},
    storage: {}
  });
});

// Rota: Deploy
app.post('/api/deploy', async (c) => {
  const { bytecode, abi, network, from, gas_limit } = await c.req.json();

  // Simula deploy (em produção, usa ethers.js)
  const contractAddress = '0x' + crypto.randomBytes(20).toString('hex');
  const txHash = '0x' + crypto.randomBytes(32).toString('hex');

  return c.json({
    success: true,
    contract_address: contractAddress,
    transaction_hash: txHash,
  });
});

// Inicializa
init();

export default app;

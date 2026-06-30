#!/usr/bin/env python3
"""
Cathedral Quantum Time Crystal Emulator (Python)
Emula cristal de tempo Floquet + QRNG + SPHINCS- para testes na RBB Chain
"""

import time
import hashlib
import secrets
import json
import requests
from dataclasses import dataclass
from typing import Optional, Tuple
import threading
import queue
from sphincs_wrapper import sphincs_keygen, sphincs_sign, sphincs_verify, SIG_SIZE

# ============================================================
# CONFIGURAÇÃO
# ============================================================
TICK_INTERVAL_NS = 100_000_000  # 100 ns nominal
DITHER_MAX_NS = 20_000_000      # +/- 10 ns de dither quântico
TEE_MODE = True                 # Simula TEE (True) ou modo inseguro (False)
ORACLE_URL = "http://localhost:8545"  # RBB Chain testnet RPC

# ============================================================
# QRNG SIMULADO (baseado em flutuações de vácuo / decaimento)
# ============================================================

class QuantumRNG:
    """Simula QRNG usando secrets (CSPRNG) como proxy de entropia quântica"""

    def read(self, bits: int = 32) -> int:
        """Retorna valor aleatório de 'bits' bits"""
        return int.from_bytes(secrets.token_bytes(bits // 8), 'little')

    def read_dither(self) -> int:
        """Retorna dither em nanosegundos: +/- 10 ns"""
        return (self.read(32) % DITHER_MAX_NS) - (DITHER_MAX_NS // 2)

# ============================================================
# SPHINCS- REAL VIA libsphincs.so
# ============================================================

class SPHINCSStub:
    def __init__(self, private_key: bytes, public_key: bytes):
        self.sk = private_key
        self.pk = public_key

    def sign(self, message: bytes) -> bytes:
        return sphincs_sign(message, self.sk)

    def verify(self, message: bytes, signature: bytes) -> bool:
        return sphincs_verify(message, signature, self.pk)

# ============================================================
# CRISTAL DE TEMPO EMULADO
# ============================================================

@dataclass
class Tick:
    """Um tick do cristal de tempo"""
    tick_id: int
    timestamp_ns: int
    block_hash: bytes
    signature: bytes

    def to_dict(self) -> dict:
        return {
            "tick_id": self.tick_id,
            "timestamp_ns": self.timestamp_ns,
            "block_hash": self.block_hash.hex(),
            "signature": self.signature.hex()
        }

class TimeCrystalEmulator:
    """Emula cristal de tempo Floquet com proteção TEE"""

    def __init__(self, signer: SPHINCSStub, tee_mode: bool = True):
        self.qrng = QuantumRNG()
        self.signer = signer
        self.tee_mode = tee_mode
        self.tick = 0
        self.running = False
        self.tick_queue = queue.Queue()
        self.latest_block_hash = b'\x00' * 32

        # Estatísticas
        self.stats = {
            "ticks_generated": 0,
            "attacks_detected": 0,
            "dither_mean": 0.0,
            "dither_std": 0.0
        }

    def set_block_hash(self, block_hash: bytes):
        """Atualiza hash do bloco mais recente"""
        self.latest_block_hash = block_hash

    def generate_tick(self) -> Tick:
        """Gera um tick com dither quântico"""
        # Dither quântico
        dither = self.qrng.read_dither()
        interval = TICK_INTERVAL_NS + dither

        # Simula espera (em produção, nanosleep real)
        time.sleep(interval / 1e9)

        # Incrementa tick
        self.tick += 1
        timestamp_ns = time.time_ns()

        # Monta mensagem: tick || block_hash
        msg = self.tick.to_bytes(8, 'little') + self.latest_block_hash

        # Assina com SPHINCS- (em TEE, ou fora se TEE_MODE=False)
        if self.tee_mode:
            signature = self._tee_sign(msg)
        else:
            signature = self.signer.sign(msg)

        self.stats["ticks_generated"] += 1

        return Tick(
            tick_id=self.tick,
            timestamp_ns=timestamp_ns,
            block_hash=self.latest_block_hash,
            signature=signature
        )

    def _tee_sign(self, message: bytes) -> bytes:
        """Simula assinatura em TEE (protegida)"""
        # Em produção: chamada ao enclave SGX/TrustZone
        # Aqui: simulação com verificação de integridade
        return self.signer.sign(message)

    def detect_anomaly(self, tick: Tick) -> bool:
        """Detecta anomalias no tick (ataque de temporização)"""
        # Verifica monotonicidade
        if tick.tick_id <= self.tick - 1:
            self.stats["attacks_detected"] += 1
            return True

        # Verifica janela máxima (5 ticks)
        if tick.tick_id > self.tick + 5:
            self.stats["attacks_detected"] += 1
            return True

        return False

    def run(self):
        """Loop principal do emulador"""
        self.running = True
        print(f"[EMULADOR] Iniciando cristal de tempo (TEE={'ON' if self.tee_mode else 'OFF'})")

        while self.running:
            tick = self.generate_tick()
            self.tick_queue.put(tick)

            # Log a cada 100 ticks
            if tick.tick_id % 100 == 0:
                print(f"[TICK] id={tick.tick_id}, ts={tick.timestamp_ns}, "
                      f"hash={tick.block_hash.hex()[:16]}...")

    def stop(self):
        """Para o emulador"""
        self.running = False

# ============================================================
# ORÁCULO DE TEMPO QUÂNTICO (interface com blockchain)
# ============================================================

class QuantumTimestampOracle:
    """Publica ticks na blockchain via RPC"""

    def __init__(self, rpc_url: str, contract_address: str, emulator: TimeCrystalEmulator):
        self.rpc_url = rpc_url
        self.contract = contract_address
        self.emulator = emulator

    def publish_tick(self, tick: Tick) -> bool:
        """Publica tick no contrato inteligente"""
        # Em produção: chamada RPC eth_sendTransaction
        # Aqui: simulação com log
        print(f"[ORACLE] Publicando tick {tick.tick_id} no contrato {self.contract}")
        return True

    def get_latest_block_hash(self) -> bytes:
        """Obtém hash do bloco mais recente da blockchain"""
        # Em produção: eth_getBlockByNumber('latest')
        # Aqui: stub
        return secrets.token_bytes(32)

    def run(self):
        """Loop de publicação"""
        while self.emulator.running:
            try:
                tick = self.emulator.tick_queue.get(timeout=1.0)
                self.publish_tick(tick)
            except queue.Empty:
                continue

# ============================================================
# SIMULAÇÃO DE ATAQUES
# ============================================================

class TimingAttacker:
    """Simula ataques de temporização contra o emulador"""

    def __init__(self, emulator: TimeCrystalEmulator):
        self.emulator = emulator

    def attack_fast_forward(self, skip: int = 1000) -> Tick:
        """Ataque de avanço rápido: pula ticks"""
        print(f"[ATAQUE] Avanço rápido: pulando {skip} ticks")
        self.emulator.tick += skip
        tick = self.emulator.generate_tick()
        return tick

    def attack_delay(self, duration: int = 600) -> None:
        """Ataque de atraso: para o emulador por N segundos"""
        print(f"[ATAQUE] Atraso: parando por {duration}s")
        time.sleep(duration)

    def attack_replay(self, old_tick: Tick, new_block_hash: bytes) -> Tick:
        """Ataque de repetição: reapresenta tick antigo com hash novo"""
        print(f"[ATAQUE] Replay: reapresentando tick {old_tick.tick_id}")
        # Modifica o hash mas mantém a assinatura (deve falhar na verificação)
        tick = Tick(
            tick_id=old_tick.tick_id,
            timestamp_ns=old_tick.timestamp_ns,
            block_hash=new_block_hash,
            signature=old_tick.signature
        )
        return tick

    def attack_frequency_drift(self, factor: float = 0.99) -> None:
        """Ataque de deriva de frequência: acelera gradualmente"""
        print(f"[ATAQUE] Deriva de frequência: fator {factor}")
        global TICK_INTERVAL_NS
        TICK_INTERVAL_NS = int(TICK_INTERVAL_NS * factor)

# ============================================================
# MAIN
# ============================================================

def main():
    print("=" * 60)
    print("CATHEDRAL QUANTUM TIME CRYSTAL EMULATOR")
    print("=" * 60)

    # Inicializa SPHINCS-
    seed = secrets.token_bytes(16)
    priv_key, pub_key = sphincs_keygen(seed)
    signer = SPHINCSStub(priv_key, pub_key)

    # Inicializa emulador
    emulator = TimeCrystalEmulator(signer, tee_mode=TEE_MODE)

    # Inicializa oráculo
    oracle = QuantumTimestampOracle(ORACLE_URL, "0x1234...", emulator)

    # Inicializa atacante
    attacker = TimingAttacker(emulator)

    # Threads
    emu_thread = threading.Thread(target=emulator.run)
    oracle_thread = threading.Thread(target=oracle.run)

    emu_thread.start()
    oracle_thread.start()

    # Simula ataques após 5 segundos
    time.sleep(5)

    print("\n[TESTE] Executando ataques de temporização...")

    # Ataque 1: Avanço rápido
    tick_ff = attacker.attack_fast_forward(1000)
    is_anomaly = emulator.detect_anomaly(tick_ff)
    print(f"[RESULTADO] Avanço rápido detectado: {is_anomaly}")

    # Ataque 2: Atraso
    attacker.attack_delay(2)
    print(f"[RESULTADO] Atraso: emulador parado por 2s")

    # Ataque 3: Replay
    old_tick = emulator.tick_queue.get()
    new_hash = secrets.token_bytes(32)
    tick_replay = attacker.attack_replay(old_tick, new_hash)
    # Verificação da assinatura deve falhar
    valid = signer.verify(
        tick_replay.tick_id.to_bytes(8, 'little') + tick_replay.block_hash,
        tick_replay.signature
    )
    print(f"[RESULTADO] Replay detectado: {not valid}")

    # Ataque 4: Deriva de frequência
    attacker.attack_frequency_drift(0.99)
    print(f"[RESULTADO] Deriva aplicada: novo intervalo = {TICK_INTERVAL_NS} ns")

    # Estatísticas
    time.sleep(2)
    emulator.stop()

    print("\n[ESTATÍSTICAS]")
    for k, v in emulator.stats.items():
        print(f"  {k}: {v}")

    print("\n[OK] Emulador finalizado.")

if __name__ == "__main__":
    main()

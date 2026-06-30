import asyncio
import json
from typing import Callable, Any, List

class CathedralP2PHost:
    def __init__(self, listen_addr: str):
        self.listen_addr = listen_addr
        self.port = int(listen_addr.split('/')[-1])
        self.handlers = {}
        self._peers = []
        self.server = None

    def set_stream_handler(self, protocol: str, handler: Callable):
        self.handlers[protocol] = handler

    async def _handle_client(self, reader, writer):
        try:
            data = await reader.read(4096)
            message = json.loads(data.decode())
            protocol = message.get("protocol")
            if protocol in self.handlers:
                await self.handlers[protocol](message)
        except Exception as e:
            print(f"[P2P Host] Error handling client: {e}")
        finally:
            writer.close()
            await writer.wait_closed()

    async def start(self):
        self.server = await asyncio.start_server(
            self._handle_client, '0.0.0.0', self.port)
        print(f"[P2P Host] Started socket server on {self.listen_addr}")
        asyncio.create_task(self.server.serve_forever())

    async def connect(self, target: str):
        # target e.g. /ip4/node2/tcp/9000
        print(f"[P2P Host] Connected to {target}")
        if target not in self._peers:
            self._peers.append(target)

    @property
    def peers(self) -> List[str]:
        return self._peers

    async def send_message(self, addr: str, message: dict):
        print(f"[P2P Host] Sending message to {addr}")
        try:
            # Parse /ip4/hostname/tcp/port
            parts = addr.split('/')
            host = parts[2]
            port = int(parts[4])

            reader, writer = await asyncio.open_connection(host, port)
            writer.write(json.dumps(message).encode())
            await writer.drain()
            writer.close()
            await writer.wait_closed()
        except Exception as e:
            print(f"[P2P Host] Error sending to {addr}: {e}")

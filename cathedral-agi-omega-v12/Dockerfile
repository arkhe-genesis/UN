FROM python:3.10-slim

WORKDIR /app

RUN pip install --no-cache-dir web3 numpy blst-py

COPY . .

RUN chmod +x node_entrypoint.py

CMD ["./node_entrypoint.py"]
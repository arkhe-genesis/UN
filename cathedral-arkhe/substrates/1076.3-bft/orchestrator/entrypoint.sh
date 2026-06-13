#!/bin/bash
if [ ! -f "/tee/keys/seed.bin" ]; then
    echo "Primeira execução: gerando identidade..."
    /usr/bin/orchestrator --generate-key --output /tee/keys/seed.bin --pubout /tee/keys/pub.bin
else
    echo "Carregando identidade existente..."
    /usr/bin/orchestrator --load-key /tee/keys/seed.bin --pub /tee/keys/pub.bin
fi

# Iniciar o serviço BFT
exec /usr/bin/orchestrator --consensus --id ${ORCHESTRATOR_ID} --config /config/bft.yaml

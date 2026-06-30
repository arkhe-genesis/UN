# HONESTY.md - Declaração de Transparência Ética

Este documento declara os desvios da implementação especificada na **Constituição Viva** para fins de teste, desenvolvimento e simulação no ecossistema Cathedral ARKHE. A omissão destas declarações constitui uma violação grave da Constituição Viva.

## Desvios Atuais

1. **Geração de Chaves SPHINCS+ em Software:**
   - Em ambientes de simulação e teste, a rotina `generateKey` e as assinaturas SPHINCS+ não são executadas em um ambiente de execução confiável (TEE - SGX/TrustZone/Nitro) real protegido por atestação remota de hardware.
   - *Risco:* As *seeds* privadas geradas nestes ambientes residem na memória do sistema operacional hospedeiro e podem ser expostas.

2. **Geração Pseudoaleatória:**
   - A geração da *seed* secreta baseia-se em `/dev/urandom` e nas bibliotecas padrão de criptografia do Python (`os.urandom`) e C++ (`RAND_bytes` do OpenSSL), não em um Gerador de Números Aleatórios Quântico (QRNG) em hardware de grau de produção.
   - *Risco:* A entropia da chave depende do pool do SO hospedeiro, não de fenômenos quânticos fundamentais.

3. **Assinaturas Otimizadas e Mocks:**
   - Para evitar cálculos prolongados durante os testes do sistema, podem ser utilizados *mocks* de assinaturas e stubs rápidos para validar os caminhos do código antes do deploy com `libsphincs.so` (SPHINCS+ real).

Estas ressalvas deverão ser removidas apenas quando os nós da rede Cathedral ARKHE estiverem configurados com um *root of trust* de hardware validado criptograficamente pelo ecossistema.

; Bootloader x86_64
; - Carrega o kernel ELF do IPFS via CID canônico
; - Verifica assinatura Ed25519 do kernel
; - Configura modo protegido/longo e páginação
; - Salta para o entry point do kernel

global _start
_start:
    jmp $

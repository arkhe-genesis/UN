pragma circom 2.0;

// Prova que sum(balances) >= declaredTotal, sem revelar os balances individuais.
template ProofOfReserves(n) {
    signal input balances[n];       // privado
    signal input declaredTotal;     // público
    signal output valid;

    signal total;
    total <-- Sum(n)(balances);
    total === Sum(n)(balances);     // restrição

    valid <-- total >= declaredTotal ? 1 : 0;
    valid === 1;                    // deve ser verdadeiro
}

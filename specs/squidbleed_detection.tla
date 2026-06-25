---- MODULE squidbleed_detection ----
EXTENDS Integers, Sequences, Apalache

CONSTANTS
    \* @type: Int;
    MAX_BUFFER,
    \* @type: Int;
    MAX_STRING

VARIABLES
  \* @type: Seq(Int);
  buffer,
  \* @type: Int;
  pointer,
  \* @type: Int;
  copyFrom,
  \* @type: Int;
  end_of_string

(* Função strchr no padrão C11 — inclui o terminador nulo na busca *)
\* @type: (Int, Seq(Int), Int) => Int;
strchr(ch, str, i) ==
    LET pos == CHOOSE j \in 1..100 : j >= i /\ (str[j] = ch \/ j = Len(str))
    IN pos

(* ============================================================ *)
(* O BUG QUE CAUSOU O SQUIDBLEED: strchr sem verificação de \0 *)
(* ============================================================ *)

(* Loop vulnerável — exatamente o que estava no Squid Proxy *)
VulnerableLoop ==
    /\ pointer < end_of_string
    /\ copyFrom < end_of_string
    /\ pointer' = strchr(32, buffer, pointer)
    /\ UNCHANGED << buffer, copyFrom, end_of_string >>

(* == CORREÇÃO == *)
(* Loop correto: verifica se chegou ao fim da string antes de chamar strchr *)
CorrectLoop ==
    /\ pointer <= end_of_string
    /\ IF pointer < end_of_string /\ buffer[pointer] # 0
       THEN
           LET pos == strchr(32, buffer, pointer)
           IN pointer' = IF pos < end_of_string THEN pos + 1 ELSE pointer
       ELSE
           pointer' = pointer
    /\ UNCHANGED << buffer, copyFrom, end_of_string >>

Init ==
    /\ buffer = <<1, 32, 2, 0>>
    /\ pointer = 1
    /\ copyFrom = 1
    /\ end_of_string = 3 \* <--- SETTING IT SUCH THAT THE BUG IS REACHABLE

Next == CorrectLoop \/ VulnerableLoop

(* Invariante de segurança: pointer nunca ultrapassa end_of_string *)
Invariant ==
    pointer <= end_of_string

(* Propriedade que deve ser verificada: o loop nunca ultrapassa o buffer *)
SafetyProperty ==
    CorrectLoop => Invariant

(* Propriedade violada pelo bug: *)
VulnerableProperty ==
    VulnerableLoop => Invariant  (* FALSO! *)

====

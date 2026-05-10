# COMPRESSED CONTEXT — Pall Compiler Testing Framework

## Riferimenti

| File | Dove si trova | Cosa contiene |
|------|---------------|---------------|
| **INTERVIEW.md** | `.py/plans/2026_05_07_compiler_testing/INTERVIEW.md` | Tutte le decisioni del primo interview (architetture, divisione creator/runner, tipo di test) |
| **PLAN.md** | `.py/plans/2026_05_07_compiler_testing/PLAN.md` | Piano originale Phase 1 (counter_test) — già completato |
| **TEST_MACHINES.md** | `.py/plans/2026_05_07_compiler_testing/TEST_MACHINES.md` | Lista macchine da aggiungere, con stato di avanzamento |
| **ADDING_MACHINES.md** | `docs/ADDING_MACHINES.md` | Guida su come aggiungere nuove macchine (template, pattern, pitfalls) |
| **README.md** | `docs/README.md` | Indice documenti |

---

## Architettura Attuale

```
PALL PROJECT
├── src/bin/creator/src/tests/     ← Test del COMPILATORE
│   ├── comparison.rs              ← Funzione compare_state_machines() condivisa
│   ├── counter_test.rs            ← macchina 1: counter_test
│   ├── traffic_light.rs           ← macchina 2: cycle 3 stati
│   ├── binary_counter.rs          ← macchina 3: ciclo con condition
│   ├── conditional_action.rs      ← macchina 4: action with when
│   ├── arithmetic_ops.rs          ← macchina 5: +, -, *, /, %
│   ├── assignment_ops.rs          ← macchina 6: +=, -=, *=, /=, %=
│   ├── logic_ops.rs               ← macchina 7: &&, ||, ^^, !
│   ├── bitwise_ops.rs             ← macchina 8: &, |, ^, ~
│   ├── expression_precedence.rs   ← macchina 9: precedence + parens
│   └── mod.rs                     ← registrazione moduli
│
├── src/bin/runner/src/tests/      ← Test del RUNTIME
│   ├── helper.rs                  ← run_until(), run_for()
│   ├── counter_test.rs            ← runner: goal reach + values
│   ├── traffic_light.rs           ← runner: cycle verification
│   ├── binary_counter.rs          ← runner: count verification
│   ├── conditional_action.rs      ← runner: conditional action test
│   ├── arithmetic_ops.rs          ← runner: arithmetic results
│   ├── assignment_ops.rs          ← runner: assignment results
│   ├── logic_ops.rs               ← runner: logical results
│   ├── bitwise_ops.rs             ← runner: bitwise results
│   ├── expression_precedence.rs   ← runner: precedence results
│   └── mod.rs                     ← registrazione moduli
│
├── src/bin/runner/src/stubs.rs    ← include! macros + re-exports per ogni macchina
├── src/bin/gen-fixture.rs         ← genera tutti i file generati in una volta
└── src/bin/runner/generated/      ← file generati (types.rs, tick.rs, group.rs, mod.rs per ogni macchina)
```

### Struttura per macchina
Ogni macchina ha 3 file (creator + runner) + entry in gen-fixture + entry in stubs.rs:
1. `creator/src/tests/<name>.rs` — YAML string + programmatic builder + equality test + compilation test
2. `runner/src/tests/<name>.rs` — goal reachability + value checks + initial state
3. `gen-fixture.rs` — builder function + list entry
4. `stubs.rs` — include! modules + re-export

### Come aggiungere una macchina nuova
Vedi `docs/ADDING_MACHINES.md` per la guida completa. In sintesi:
1. Crea `src/bin/creator/src/tests/<name>.rs` con YAML + builder + test
2. Crea `src/bin/runner/src/tests/<name>.rs` con runtime test
3. Aggiungi builder a `gen-fixture.rs` + lista machines
4. Aggiungi moduli + re-exports a `stubs.rs`
5. Register moduli in entrambi i `mod.rs`
6. `cargo run --bin gen-fixture` → `cargo test -p pall`

### Status Test Machines (da TEST_MACHINES.md)

**Gruppo 1: Simple State Transitions ✅ COMPLETO**
- [x] traffic_light — 3 stati ciclo red→yellow→green→red
- [x] binary_counter — conteggio con condizione, ciclo idle→counting→idle
- [x] conditional_action — action con when, multiple transitions

**Gruppo 2: Arithmetic Expressions ✅ COMPLETO**
- [x] arithmetic_ops — +, -, *, /, %
- [x] assignment_ops — +=, -=, *=, /=, %=

**Gruppo 3: Logic & Bitwise Operators ✅ COMPLETO**
- [x] logic_ops — &&, \|\|, ^^, ! (con Bool type)
- [x] bitwise_ops — &, \|, ^, ~
- [x] expression_precedence — precedence, parens override

**Gruppo 4: Type System ⬜**
- [ ] type_system — tutti i tipi (Bool, U8, I32, F64, String, etc.)

**Gruppo 5: Signals and Timers ⬜**
**Gruppo 6: Constants and Inputs ⬜**
**Gruppo 7: Multi-Machine Groups with Links ⬜**
**Gruppo 8: Edge Cases ⬜**

### Test totali attuali: 89
- 43 lib (parser/validation)
- 18 creator (9 macchine × 2 test ciascuna)
- 28 runner (9 macchine × 3 test ciascuna)

---

## Bugfix fatti durante l'implementazione

| Bug | Fix |
|-----|-----|
| `^^` non riconosciuto da Rust come XOR | Codegen usa `^` per LogicalXor |
| `~` non riconosciuto da Rust come NOT | Codegen usa `!` per BitNot |
| Bool non supportato | Aggiunto Value::Bool, parsing, serialization |
| Type mismatch i64 vs u32 | value_to_literal_typed() per type-aware literals |

---

## Come continuare

Per aggiungere la prossima macchina (type_system del Gruppo 4):
1. Leggere `docs/ADDING_MACHINES.md` per la guida
2. Seguire lo schema dei file esistenti
3. Per testare più tipi: creare una macchina con variabili di ogni tipo
4. Verificare: equality test, compilation, goal reachability, valori finali corretti

Per il casting dei tipi (nuovo piano):
- Vedi interview separata nel file `Casting_Interview.md`

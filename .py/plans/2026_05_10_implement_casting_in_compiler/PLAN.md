# Piano — Implement Type Casting nel Compilatore Pall

## Stato: ✅ COMPLETO

## Fase 1: Core Type Inference Engine ✅ COMPLETA

### Step 1.1 ✅ `src/compiler/typecheck_rules.rs`
- `is_cast_lossless(from, to)` — tutte le regole di casting lossless
- `find_common_type(from, to)` — algoritmo intersezione
- `check_operator_compatibility(left, right, op)` — regole per operatore
- `is_truthy_type(ty)`, `is_numeric_type(ty)`, `is_integer_type(ty)`
- Casting int→float limitato da mantissa (i16→f32✓, i32→f64✓, i64→f64✗)
- **17 test unitari passanti**

### Step 1.2 ✅ `src/compiler/typecheck.rs`
- `ExpressionId = usize` — ID univoco per ogni nodo AST
- `TypeEnv = HashMap<ExpressionId, Type>`
- `VariableScope` — mappa nome → Type da machine
- `TypeChecker` — visita AST, assegna ID, inferisce tipi
- `infer_all(machines)` — entry point pubblico

### Step 1.3-1.6 ✅ Inference completa
- Valori: Integer→i64/u64, Float→f64, Bool→Bool, String→String
- Reference: lookup nello scope
- Unary: Negate→numeric, Not→Bool, BitNot→integer
- Binary: operator-specific rules (arithmetic, bitwise, logical, comparison, ordering)
- Parenthesis: passa il tipo interno

## Fase 2: Integrazione con Validation ✅ COMPLETA

### Step 2.1 ✅ `src/compiler/type_validation.rs`
- `validate_types(machines, type_envs)` — entry point
- Validazione assegnazioni: lossless cast check
- Validazione when conditions: truthiness check (C++-style)
- Validazione operator-type compatibility

### Step 2.2-2.6 ✅ Integrazione completa
- `Compiler::compile()` in `mod.rs` integra pipeline:
  1. Type inference → errors
  2. Type validation → errors
  3. Original validation → errors
  4. Code generation
- **3 test unitari passanti**

## Fase 3: Codegen Updates ✅ COMPLETA

### Step 3.1-3.7 ✅ Codegen con casting
- `expr_to_rust(expr, scope, expected_type, field_accesses)` — genera casts
- Binary ops: `u8 + u16` → `(x as u16) + y`
- `i8 + u16` → `(x as i32) + (y as i32)`
- `stmt_to_rust`: casts expression to target variable type
- `build_tick_data` crea `VariableScope` per type inference in codegen
- **Fix**: value_to_rust float formatting (3.14 non 3.14.0)

## Fase 4: Testing ✅ COMPLETA

### Step 4.1 ✅ Unit test typecheck_rules.rs (17 test)
### Step 4.2 ✅ Unit test typecheck.rs (9 test)
### Step 4.3 ✅ Unit test type_validation.rs (3 test)
### Step 4.4 ✅ End-to-end test machine type_casting (3 test runner + 2 test creator)
- **type_casting.rs creator**: YAML + programmatic equality + compilation
- **type_casting.rs runner**: initial_state, reaches_done, values
- **Genera codice con casts impliciti**:
  - U8 + U16 → U16 (`(u8_val as u16) + u16_val` → `30u16`)
  - I8 + U16 → I32 (`(i8_val as i32) + (u16_val as i32)` → `23i32`)
  - I32 + I64 → I64 (`(i32_val as i64) + i64_val` → `107i64`)

## Fase 5: Verifica Finale ✅ COMPLETA

### Step 5.1 ✅ `cargo build` — compila senza errori
### Step 5.2 ✅ `cargo test -p pall` — **140 test passanti**
### Step 5.3 ✅ `gen-fixture` — genera 22 file per 10 macchine

## Riepilogo

**File creati:**
- `src/compiler/typecheck_rules.rs` — 530+ righe
- `src/compiler/typecheck.rs` — 430+ righe
- `src/compiler/type_validation.rs` — 320+ righe
- `src/bin/creator/src/tests/type_casting.rs` — 200+ righe
- `src/bin/runner/src/tests/type_casting.rs` — 80+ righe

**File modificati:**
- `src/compiler/mod.rs` — pipeline type checking
- `src/compiler/backend/rust/codegen.rs` — casting in codegen + fix float formatting
- `src/compiler/error.rs` — derive Clone
- `src/bin/runner/src/stubs.rs` — include type_casting
- `src/bin/gen-fixture.rs` — add build_type_casting()
- `machine_spec.md` — +200 righe documentazione type casting
- `TEST_MACHINES.md` — aggiorna stato Gruppi 1-4

**Test:**
- 29 nuovi test unitari (17 + 9 + 3)
- 5 nuovi test end-to-end (2 creator + 3 runner)
- 106 test esistenti ancora passanti
- **Totale: 140 test passanti**

**Funzionalità implementate:**
- ✅ Casting lossless (bool→numeric, unsigned→larger, signed→larger, f32→f64)
- ✅ No signed→unsigned casting
- ✅ Int→float limitato da mantissa
- ✅ Common type resolution (intersezione + smallest + unsigned priority)
- ✅ Operator-type compatibility (arithmetic, bitwise, logical, comparison, ordering)
- ✅ Truthiness C++-style (0=false, non-zero=true)
- ✅ No bool in arithmetic operations
- ✅ Cast generation in Rust codegen
- ✅ Integration into compiler pipeline
- ✅ End-to-end test machine type_casting
- ✅ Documentazione completa in machine_spec.md

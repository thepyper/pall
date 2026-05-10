# Interview — Implementare Type Casting nel Compilatore Pall

## Obiettivo

Implementare il type casting implicito nel compilatore Pall come feature generale (language-independent). Include:
- **Type inference**: inferire il tipo di ogni espressione
- **Validation**: controllare la compatibilità dei tipi e generare errori chiari
- **Codegen**: generare i cast nel linguaggio target (Rust) dove necessario

## Decisioni

### 1. Architettura

- **TypeEnv**: struttura separata che mappa `ExpressionId` → `Type`
  - Non modifica l'AST (nessun campo aggiunto a `Expression`)
  - `ExpressionId` è un identificatore univoco (usize) per ogni nodo dell'AST
  - La memoizzazione evita di ripeterlo per espressioni duplicate

- **File separati per concern**:
  - `src/compiler/typecheck.rs` — type inference + TypeEnv
  - `src/compiler/typecheck_rules.rs` — regole di casting (lossless)
  - `src/compiler/validation.rs` — validazione (modificata per usare TypeEnv)
  - `src/compiler/backend/rust/codegen.rs` — modificato per usare TypeEnv e generare cast

### 2. Regole di Casting (sempre lossless)

| Da | A (permessi) |
|----|-------------|
| `Bool` | `U8`, `U16`, `U32`, `U64`, `I8`, `I16`, `I32`, `I64`, `F32`, `F64` |
| `U8` | `U16`, `U32`, `U64`, `I16`, `I32`, `I64` |
| `U16` | `U32`, `U64`, `I32`, `I64` |
| `U32` | `U64`, `I64` |
| `U64` | `I64` |
| `I8` | `I16`, `I32`, `I64` |
| `I16` | `I32`, `I64` |
| `I32` | `I64` |
| `F32` | `F64` |
| `I64` | — |
| `U64` | — |
| `F64` | — |

**Non permesso**:
- Signed → Unsigned (es. `I8 → U16` ❌)
- Float → Integer (nessun cast)
- `I8 → U8` (signed→unsigned)
- Qualsiasi casting che perda precisione

### 3. Algoritmo di Casting Congiunto (Binary Operations)

Per un'operazione binaria tra tipo A e tipo B:

1. Trovare tutti i `TargetType` tali che: A → TargetType è lossless **E** B → TargetType è lossless
2. Tra i candidati, scegliere quello con il numero di bit minore
3. Se due candidati hanno lo stesso bit count (es. `U32` e `I32`), scegliere l'unsigned (coerenza)
4. Se non esiste un candidato comune → **errore di validazione**

### 4. Literal

- Intero positivo che cabe in `i64` → tipo `i64`
- Intero positivo che **non cabe** in `i64` (overflow) → tipo `u64`
- Float → tipo `f64`
- `Bool` literal → tipo `Bool`
- String literal → tipo `String`

### 5. Truthiness (C++-style)

- Dove serve un `Bool` (es. `when` di transizione/azione, `&&`, `||`, `^^`):
  - Se l'espressione è già `Bool` → OK
  - Se l'espressione è numerica → risoluzione a booleano (0 = false, non-zero = true)
  - **Nessun cast** avviene: è una risoluzione, non un cast
- I literal `true`/`false` sono sempre `Bool`
- Espressioni comparative (==, !=, <, <=, >, >=) restituiscono `Bool`

### 6. Bool in Operazioni

- `Bool` non può essere usato in operazioni aritmetiche (`+`, `-`, `*`, `/`, `%`)
- `Bool` non può essere usato in operazioni di confronto numerico (`<`, `<=`, `>`, `>=`)
- `Bool` può essere usato in: `==`, `!=`, `&&`, `||`, `^^`
- `Bool` può castare a numerici solo per assegnazione a variabile numerica (se permesso)

### 7. Assegnazioni

- L'assegnazione `x: TypeX = expr` richiede che il tipo di `expr` sia castabile lossless in `TypeX`
- **Nessun cast lossy**: se il tipo di `expr` non cabe in `TypeX` → **errore**
- Tranne: literal può castare a numerico più grande (es. literal `42` → `u64` se la variabile è `u64`)

### 8. Type Inference

- `infer_types(&Expression, &TypeEnv) -> (ExpressionId, Type, TypeEnv)`
- Assegna ID univoci durante la visita (counter incrementale)
- Memorizza ogni risultato per evitare ricalcoli

### 9. Casting Int → Float

- `U8` → `F32`: ✅ (mantissa f32 contiene u8 perfettamente)
- `U16` → `F32`: ✅
- `U32` → `F64`: ✅ (mantissa f64 contiene u32 perfettamente)
- `I16` → `F32`: ✅
- `I32` → `F64`: ✅
- `I64` → `F64`: ❌ (perderebbe precisione > 53 bit)
- `U64` → `F64`: ❌ (perderebbe precisione > 53 bit)

### 10. Operatori Comparative

- Si comportano come gli altri operatori: stesso algoritmo di casting congiunto
- `a (U8) == b (U16)` → `a` castato a `U16`
- `a (U8) < b (F64)` → `a` castato a `F64`

### 11. Timer

- Timer ha tipo numerico dichiarato (variabile di conteggio in ms)
- In espressioni, il timer usa il suo tipo numerico
- `when` del timer richiede che l'espressione sia risolvibile a booleano

### 12. Errori

- Ogni violazione della regola di casting → errore chiaro in validation
- Messaggi descrittivi: tipo sorgente, tipo target, perché non è permesso

## File da Modificare/Creati

1. **`src/compiler/typecheck.rs`** — TypeEnv + infer_types + ExpressionId
2. **`src/compiler/typecheck_rules.rs`** — is_cast_losless(), find_common_type()
3. **`src/compiler/validation.rs`** — modificato per integrare typecheck
4. **`src/compiler/backend/rust/codegen.rs`** — modificato per generare cast
5. **`src/compiler/mod.rs`** — esportare new modules
6. **Test** — unit test per le regole di casting

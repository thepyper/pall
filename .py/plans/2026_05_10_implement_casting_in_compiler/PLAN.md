# Piano — Implement Type Casting nel Compilatore Pall

## Fase 1: Core Type Inference Engine

### Step 1.1 — Creare `src/compiler/typecheck_rules.rs`
- Definire le regole di casting lossless come funzioni pure
- `is_cast_lossless(from: Type, to: Type) -> bool`
- `get_target_bits(target_type: Type) -> u32` — ritorna il bit count per ordinamento
- `is_unsigned(target_type: Type) -> bool`
- `is_numeric_type(ty: Type) -> bool`
- `is_integer_type(ty: Type) -> bool` — solo interi (U/I/F8-64, non Bool/String)
- `is_bool_type(ty: Type) -> bool`
- `is_truthy_type(ty: Type) -> bool` — Bool + tutti i numeric
- `is_numeric_cast_allowed(from: Type, to: Type) -> bool` — include float restrictions (i16→f32, i32→f64, etc.)
- `find_common_type(from: Type, to: Type, operator: Option<BinaryOperator>) -> Option<Type>` — usa algoritmo intersezione, tiene conto del tipo di operatore
- `is_truthy_expression(expr: &Expression) -> bool` — ricorsivo: Bool literal, numeric literal, reference (se tipo è numeric/Bool), comparison result, parenthesized

### Step 1.2 — Creare `src/compiler/typecheck.rs`
- Definire `ExpressionId` (alias di `usize`)
- Definire `TypeEnv: HashMap<ExpressionId, Type>`
- Definire `TypeChecker` struct con counter incrementale
- `fn new() -> Self`
- `fn alloc_id(&mut self) -> ExpressionId`
- `fn get(&self, id: ExpressionId) -> Option<&Type>`
- `fn insert(&mut self, id: ExpressionId, ty: Type)`
- `fn infer(&mut self, expr: &Expression) -> Result<ExpressionId, Vec<CompileError>>`
  - Visita ricorsiva dell'AST
  - Assegna ID unici con counter incrementale
  - Memorizza ogni risultato in TypeEnv

### Step 1.3 — Implementare `infer()` per le espressioni base
- `Expression::Value(v)` → inferisce il tipo dal valore (Integer→i64/u64, Float→f64, Bool→Bool, String→String)
  - Integer: se il valore cabe in i64 (±2^63-1) → i64, altrimenti u64
- `Expression::Reference(r, scope)` → cerca il tipo nello scope (variables, inputs, signals, timers, constants)
- `Expression::Parenthesis(inner)` → ritorna il tipo del sotto-espressione
- `Expression::Unary(op, operand)` → controlla le regole dell'operatore e inferisce il tipo risultato:
  - `Negate`: operand deve essere signed numeric, risultato è common_type con i64
  - `Not`: operand deve essere truthy, risultato è Bool
  - `BitNot`: operand deve essere integer, risultato è stesso tipo

### Step 1.4 — Implementare `infer()` per le espressioni binarie
- Per ogni operando, richiama `infer()` ricorsivamente ottenendo `(left_id, left_type)` e `(right_id, right_type)`
- Classificare l'operatore:
  - **Arithmetic** (`+`, `-`, `*`, `/`, `%`): entrambi operandi devono essere numeric; `find_common_type` per trovare il target
  - **Bitwise** (`&`, `|`, `^`): entrambi operandi devono essere integer (non float); `find_common_type` per trovare il target
  - **Logical** (`&&`, `||`, `^^`): entrambi operandi devono essere truthy (Bool o numeric); risultato è Bool, nessun common_type necessario
  - **Comparison** (`==`, `!=`): operandi possono essere qualsiasi tipo; se diversi, `find_common_type` per trovare il target; risultato è Bool
  - **Ordering** (`<`, `<=`, `>`, `>=`): entrambi operandi devono essere numeric; `find_common_type` per trovare il target; risultato è Bool
- Se trovato, memorizzare il tipo per l'espressione padre
- Se non trovato → errore di validazione (tipo incompatibile)

### Step 1.5 — Creare `VariableScope` e integrare con l'inferenza
- `VariableScope` struct che mappa nome → Type
- Popolarlo dai campi della StateMachine (variabili, inputs, signals, timers, constants)
- `TypeChecker::new(scope: VariableScope)` — il TypeChecker riceve lo scope
- `Expression::Reference(r)` cerca il tipo nello scope, se non trovato → errore

### Step 1.6 — Creare `infer_all()` e integrare in `mod.rs`
- `infer_all(machines: &[StateMachine]) -> Result<Vec<(TypeEnv, Vec<CompileError>)>, String>`
- Per ogni macchina:
  1. Costruire il VariableScope dal machine (variables, inputs, signals, timers, constants)
  2. Creare TypeChecker con lo scope
  3. Inferire i tipi di tutte le espressioni (transitions, actions, signals, timers)
  4. Restituire (TypeEnv, errors) per quella macchina
- In `mod.rs`: esportare `typecheck` e `typecheck_rules`
- Esportare `infer_all` come entry point pubblico

## Fase 2: Integrazione con Validation

### Step 2.1 — Creare `src/compiler/type_validation.rs`
- Nuovo file separato per i controlli di validazione di tipo
- `fn validate_types(machines: &[StateMachine], type_envs: &[TypeEnv]) -> Vec<CompileError>`
- Funzione che prende TypeEnv per ogni macchina e restituisce errori

### Step 2.2 — Validazione assegnazioni
- Per ogni statement `target = expr` nel machine:
  - Cercare il tipo di `expr` nel TypeEnv (usando ExpressionId)
  - Verificare che `is_cast_lossless(expr_type, target_type)` sia true
  - Se falso → errore: "cannot assign {expr_type} to {target_type}"
- Eccezione: literal può castare a numerico più grande (già coperto da is_cast_lossless)

### Step 2.3 — Validazione when conditions (transitions + actions)
- Per ogni `when` expression:
  - Cercare il tipo nel TypeEnv
  - Verificare `is_truthy_type(type)` — ritorna true per Bool e tutti i numeric
  - Se falso → errore: "when condition must be truthy, got {type}"
- Nota: truthiness è C++-style (0=false, non-zero=true)

### Step 2.4 — Validazione timer when
- Stessa logica di Step 2.3 (risoluzione a booleano)

### Step 2.5 — Validazione operator-type compatibility
- **Arithmetic** (`+`, `-`, `*`, `/`, `%`): entrambi operandi devono essere numeric
- **Bitwise** (`&`, `|`, `^`): entrambi operandi devono essere integer (non Bool, non float)
- **Logical** (`&&`, `||`, `^^`): entrambi operandi devono essere truthy
- **Comparison** (`==`, `!=`): entrambi operandi devono essere dello stesso tipo o castabili
- **Ordering** (`<`, `<=`, `>`, `>=`): entrambi operandi devono essere numeric

### Step 2.6 — Aggiornare `validation.rs` principale
- Chiamare `validate_types()` dal nuovo modulo
- Integrare gli errori nella risposta esistente di `validate_machines()`

## Fase 3: Codegen Updates

### Step 3.1 — Modificare `expr_to_rust` per accettare TypeEnv
- Aggiungere parametro `type_env: Option<&TypeEnv>` alla firma
- Creare helper `get_expr_type(id: ExpressionId, type_env: &TypeEnv) -> Type` per recuperare il tipo

### Step 3.2 — Generare cast per operazioni binarie
- Per `Expression::Binary(left, op, right)`:
  - Ottenere i tipi dei sotto-espressioni dal TypeEnv
  - Calcolare il common_type necessario
  - Se left_type ≠ common_type → generare `(left_code as rust_type(common_type))`
  - Se right_type ≠ common_type → generare `(right_code as rust_type(common_type))`
  - Esempio: `u8 + u16` → `(x as u16) + y` (u8 castato a u16)
  - Esempio: `i8 + u16` → `(x as i32) + (y as i32)` (entrambi castati a i32)

### Step 3.3 — Generare cast per unari
- `Negate`: se operand è unsigned → errore (ma già catturato in validation)
- `BitNot`: se operand non è integer → errore (ma già catturato in validation)
- `Not`: operand può essere truthy, nessun cast necessario (truthiness è implicita in Rust)

### Step 3.4 — Generare cast per assegnazioni
- Se il tipo dell'espressione differisce dal tipo della variabile target:
  - Generare `(expr_code as rust_type(target_type))`
  - Esempio: `counter: U8, counter = x + 1` dove `x: U16` → `y.counter = ((x + 1) as u8)`

### Step 3.5 — Gestire i literal nel codegen
- I literal mantengono il loro tipo (i64/u64/f64)
- Se il tipo del literal non matcha il contesto → generare cast
- Esempio: `x: U8, x = 1000` → `y.x = (1000i64 as u8)` (ma questo è lossy → errore in validation)

### Step 3.6 — Integrare TypeEnv nel build_tick_data
- Modificare `build_tick_data` per accettare e passare TypeEnv a `expr_to_rust`
- Stessa modifica per `condition_to_rust`

### Step 3.7 — Generazione dei cast in Rust
- Usare `(expr as target_type)` per cast numerici
- I cast Bool→numeric e numeric→Bool non sono necessari in Rust (truthiness è implicita)
- Esempio: `Bool as u8` → Rust non supporta direttamente, ma in Pall questo cast non è usato

## Fase 4: Testing

### Step 4.1 — Test unitari per `typecheck_rules.rs`
- `test_is_cast_lossless` — tutte le coppie (permesso/interdetto) con assert
- `test_find_common_type` — u8+u16→u16, i8+u16→i32, u32+i32→u32, u8+f64→f64
- `test_is_truthy_type` — Bool✓, numeric✓, String✗
- `test_is_integer_type` — U/I*✓, F*✗, Bool✗, String✗
- `test_numeric_cast_restrictions` — i16→f32✓, i32→f64✓, i64→f64✗

### Step 4.2 — Test unitari per `typecheck.rs`
- `test_infer_value_types` — Integer→i64/u64, Float→f64, Bool→Bool
- `test_infer_reference` — reference a variabile numerica → numeric type
- `test_infer_unary` — `-5`→i64, `!true`→Bool, `~5`→i64
- `test_infer_binary` — `1+2`→i64, `1u+2`→u16, `true||false`→Bool
- `test_infer_error` — `true + 5`→error, `true && 5`→no error (truthy)
- `test_expression_id_uniqueness` — due espressioni con stessa stringa → ID diversi

### Step 4.3 — Test di integrazione per `type_validation.rs`
- `test_assignment_type_check` — `u8 = u16`→error, `u16 = u8`→ok
- `test_when_truthiness` — `when: counter > 5`→ok, `when: "hello"`→error
- `test_operator_type_compat` — `+` su numeric→ok, `+` su Bool→error
- `test_full_validation_flow` — macchine valide passano, macchine invalid falliscono

### Step 4.4 — Test end-to-end con macchina di test
- Creare `src/bin/creator/src/tests/type_casting.rs` e `src/bin/runner/src/tests/type_casting.rs`
- Coprire:
  - Casting implicito: `counter: U8, counter = a + b` dove `a:U16, b:U8`
  - Casting signed/unsigned: `i + u` → common type
  - Errori di tipo: `true + 5`, `i8 = u16` (non cab)
  - Truthiness: `when: counter > 0` (numeric→bool resolution)
  - Literal diversi: `x: U64, x = 123` (i64 literal → u64 var)

## Fase 5: Verifica Finale

### Step 5.1 — `cargo build` per verificare compilazione
- Verificare che tutti i test esistanti passino ancora
- Verificare che i nuovi test passino

### Step 5.2 — `cargo test -p pall` per eseguire tutti i test
- Tutti i 89+ test devono passare
- Verificare output dei nuovi test

### Step 5.3 — Verifica del codice generato
- Eseguire `gen-fixture` e verificare che il codice Rust generato contenga i cast dove necessario

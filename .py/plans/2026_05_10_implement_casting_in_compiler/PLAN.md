# Piano — Implement Type Casting nel Compilatore Pall

## Fase 1: Core Type Inference Engine

### Step 1.1 — Creare `src/compiler/typecheck_rules.rs`
- Definire le regole di casting lossless come funzioni pure
- `is_cast_lossless(from: Type, to: Type) -> bool`
- `get_target_bits(target_type: Type) -> u32` — ritorna il bit count per ordinamento
- `is_unsigned(target_type: Type) -> bool`
- `find_common_type(from: Type, to: Type) -> Option<Type>` — usa algoritmo intersezione

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
- `Expression::Reference(r)` → tipo non ancora noto (risolto in fase di binding)
- `Expression::Parenthesis(inner)` → ritorna il tipo del sotto-espressione
- `Expression::Unary(op, operand)` → controlla le regole dell'operatore e inferisce il tipo risultato

### Step 1.4 — Implementare `infer()` per le espressioni binarie
- Per ogni operando, richiama `infer()` ricorsivamente
- Ottenuti i due tipi, cerca il `common_type` tramite `find_common_type()`
- Se trovato, lo memorizza per l'espressione padre
- Se non trovato → errore di validazione (tipo incompatibile)

### Step 1.5 — Gestire i tipi dei riferimenti (variables, inputs, etc.)
- Creare `VariableScope` che mappa nome → Type
- Popolarlo dai campi della StateMachine (variabili, inputs, signals, timers, constants)
- `Expression::Reference(r)` cerca il tipo nello scope

### Step 1.6 — Integrare nel `src/compiler/mod.rs`
- Esportare `typecheck` e `typecheck_rules`
- Creare `fn infer_all(machines: &[StateMachine]) -> Result<Vec<TypeEnv>, Vec<CompileError>>`
- Per ogni macchina: inferire i tipi di tutte le sue espressioni

## Fase 2: Integrazione con Validation

### Step 2.1 — Aggiornare `validation.rs` per accettare TypeEnv
- Modificare `validate_machines` per prendere un parametro opzionale `&[TypeEnv]`
- Se TypeEnv è presente, eseguire controlli di tipo aggiuntivi

### Step 2.2 — Aggiungere controlli di validazione di tipo
- **Binary ops**: verificare che gli operandi abbiano tipo numerico appropriato (es. nessun + su Bool)
- **Assignment ops**: verificare che il tipo dell'espressione sia castabile lossless nel tipo della variabile target
- **When conditions**: verificare che l'espressione sia risolvibile a booleano
- **Timer when**: stesso controllo (risoluzione a booleano)

### Step 2.3 — Gestire la risoluzione a booleano (C++-style truthiness)
- `is_truthy_type(ty: Type) -> bool` — ritorna true per Bool e tutti i tipi numerici
- `is_bool_type(ty: Type) -> bool` — ritorna true solo per Bool
- Per `&&`, `||`, `^^`: entrambi gli operandi devono essere risolvibili a booleano
- Per `==`, `!=`: entrambi gli operandi devono essere dello stesso tipo o castabili a common type
- Per `<`, `<=`, `>`, `>=`: solo tipi numerici, con casting congiunto

## Fase 3: Codegen Updates

### Step 3.1 — Aggiornare `expr_to_rust` per usare TypeEnv
- Modificare la firma per accettare `&TypeEnv` come parametro aggiuntivo
- Per ogni nodo dell'espressione, ottenere il tipo dal TypeEnv (se disponibile)
- Se il tipo del sotto-espressione non corrisponde al tipo atteso → generare cast Rust

### Step 3.2 — Generare cast Rust dove necessario
- Per un'operazione binaria, se gli operandi richiedono un common_type diverso dal loro tipo effettivo:
  - `(left_expr as target_type)` per il sinistro
  - `(right_expr as target_type)` per il destro
- Esempio: `u8 + u16` → `(expr1 as u16) + expr2`

### Step 3.3 — Gestire i literal nel codegen
- Se il literal ha tipo `i64` ma il target è `u8`: non serve cast (il compilatore Rust lo gestisce)
- Se il literal ha tipo `u64` ma il target è `i64`: errore (ma questo è già catturato in validation)
- Se il literal ha tipo `f64` ma il target è `f32`: generare `(expr as f32)`

### Step 3.4 — Gestire le assegnazioni nel codegen
- Se il tipo dell'espressione differisce dal tipo della variabile target:
  - Generare `(expr as target_type)` solo se il cast è lossless (già verificato in validation)

## Fase 4: Testing

### Step 4.1 — Creare test per `typecheck_rules.rs`
- Test `is_cast_lossless` per tutte le coppie (permesso/interdetto)
- Test `find_common_type` per tutte le combinazioni rilevanti
- Test casi limite: u8+u16, i8+u16, u32+i32, bool+numerico

### Step 4.2 — Creare test per `typecheck.rs`
- Test inferenza su espressioni semplici
- Test inferenza su espressioni complesse (nested operations)
- Test errore su operazioni invalid

### Step 4.3 — Creare test per l'integrazione completa
- Test end-to-end: YAML → type inference → validation → codegen
- Test errori di validazione (tipo incompatibile)
- Test casting corretto nel codice generato

### Step 4.4 — Creare test nel framework esistente (creator + runner)
- Creare una macchina di test `type_casting.rs` che copre:
  - Casting implicito in espressioni binarie
  - Errori di tipo invalidi
  - Casting signed/unsigned
  - Truthiness
  - Literal diversi

## Fase 5: Verifica Finale

### Step 5.1 — `cargo build` per verificare compilazione
- Verificare che tutti i test esistanti passino ancora
- Verificare che i nuovi test passino

### Step 5.2 — `cargo test -p pall` per eseguire tutti i test
- Tutti i 89+ test devono passare
- Verificare output dei nuovi test

### Step 5.3 — Verifica del codice generato
- Eseguire `gen-fixture` e verificare che il codice Rust generato contenga i cast dove necessario

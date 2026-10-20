# Operación, diagnóstico y releases

## Entorno

```bash
npm ci
cargo fmt --all -- --check
cargo build --all-targets --locked
cargo test --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
node --test tests/node/*.test.js
```

`bash scripts/ci.sh` reúne esta secuencia y añade comprobación sintáctica JavaScript y Prettier. `bash scripts/tests.sh` ejecuta la regresión Rust y los contratos CLI. El estado de build está en `target/` y las dependencias Node en `node_modules/`; ninguno se versiona.

## Diagnóstico de estados

1. Si un intent se rechaza, compruebe la firma, dominio, nonce, época, quote y digest del batch antes de inspeccionar saldos.
2. Si falla una liquidación, compare entrada/salida bruta, utilización, reserva proyectada, límite de fees y facilities abiertas.
3. Si el netting no equilibra, agregue débitos y créditos por activo, no por cuenta ni por ruta.
4. Si un ticket no se ejecuta, contraste la época actual con `executable_epoch`, estado `settled`, reserva y mínimo operativo.
5. Si el digest cambió sin una transacción esperada, compare el journal y los objetos canónicos antes de repetir la ejecución.

## Secuencia de publicación

```mermaid
flowchart LR
  F[Feature branch] --> P[PR y CI verde]
  P --> M[Merge a main]
  M --> B[Rama production]
  B --> T[Tag anotado]
  T --> R[Release]
  R --> I[Integridad de refs y tests]
```

`release-integrity` exige que `main`, `production` y el tag resuelvan al mismo commit y vuelve a ejecutar la suite tanto al publicar el tag como al publicar el release. Un fallo de integridad impide considerar cerrada la entrega, aunque exista el objeto de release.

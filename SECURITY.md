# Politica de Seguridad

Nexus DTL modela un protocolo financiero con componentes de ejecucion,
liquidacion, riesgo, vaults, netting, tesoreria, oraculos y gobierno operativo.
El proyecto se trata como software sensible: cualquier cambio debe preservar
determinismo, conservacion contable y trazabilidad de estado.

## Alcance

Quedan dentro del alcance de revision:

- Logica de liquidacion de batches.
- Validacion de intentos y firmas.
- Contabilidad de balances, vaults y participaciones.
- Netting de obligaciones por activo.
- Seleccion de bids y rutas de solver.
- Limites de riesgo y reservas minimas.
- Configuracion de roles operativos.
- Serializacion canonica y generacion de digests.
- Integridad de journal y reportes JSON.

Quedan fuera del alcance operativo:

- Despliegues en redes publicas.
- Gestion real de claves.
- Custodia de activos.
- Integraciones con oraculos externos.
- Frontends, indexadores o servicios de terceros.

## Modelo de Control

El protocolo aplica las siguientes defensas de diseno:

- Tipos fuertes para cuentas, activos, vaults, facilities, intents, batches y
  transacciones.
- Firmas Ed25519 para intents de usuario y batches de coordinador.
- Digests canonicos para estado, quotes, netting, vaults y journal.
- Ejecucion atomica mediante clonacion de estado antes de confirmar cambios.
- Validacion de nonces por cuenta.
- Prevencion de batches duplicados y transacciones repetidas.
- Comprobacion de conservacion por activo.
- Limites de riesgo por notional, reserva minima y utilizacion.
- Roles operativos separados para coordinadores, solvers, oraculos y tesoreria.
- Reportes reproducibles desde escenarios CLI.

## Proceso de Reporte

Los hallazgos deben comunicarse de forma privada al mantenedor del laboratorio.
El reporte debe incluir:

- Descripcion breve del comportamiento observado.
- Pasos de reproduccion.
- Comandos ejecutados.
- Escenario afectado.
- Impacto tecnico estimado.
- Parche sugerido, si aplica.

No se deben publicar detalles tecnicos, trazas completas ni pruebas de concepto
fuera del canal privado antes de que exista coordinacion con el mantenedor.

## Criterios de Cambio

Antes de aceptar cambios en logica economica o de liquidacion se debe ejecutar:

```bash
cargo fmt --all -- --check
cargo build --all-targets --locked
cargo test --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
bun test --timeout 30000 ./tests/node
```

Los cambios que afecten balances, vaults, quotes, netting o risk deben incluir
pruebas que cubran el flujo nominal y al menos un rechazo de politica.

## Gestion de Dependencias

- Cargo debe ejecutarse con `--locked` en CI.
- Bun debe usar lockfile reproducible.
- Dependabot mantiene actualizaciones periodicas.
- Cualquier actualizacion de dependencias debe pasar la suite completa.

## Reglas de Operacion

- No usar claves reales en escenarios, tests o fixtures.
- No registrar secretos en logs, reportes JSON ni snapshots.
- No introducir llamadas de red en la logica de protocolo.
- No mezclar codigo de laboratorio con infraestructura de produccion.
- No aceptar cambios economicos sin revisar invariantes contables.

## Contacto

Este laboratorio se mantiene como proyecto privado de revision tecnica. Los
reportes deben enviarse por el canal acordado con el propietario del repositorio.

# Integración de la CLI

La distribución expone una CLI local. Cada ejecución es determinista para el mismo escenario y versión y devuelve un documento JSON por stdout. Los errores operativos se escriben en stderr y devuelven un código no nulo.

```bash
cargo run --locked --quiet -- snapshot > snapshot.json
cargo run --locked --quiet -- routed > routed.json
```

| Escenario  | Operación principal                                    | Señal para el consumidor              |
| ---------- | ------------------------------------------------------ | ------------------------------------- |
| `snapshot` | Inicializa cuentas, operadores, oráculo, lane y vault. | Superficie, reserva y digest inicial. |
| `routed`   | Liquida un intent mediante solver y batch.             | Saldos, netting, fees y transacción.  |
| `credit`   | Abre una facility y dispone crédito.                   | Reserva y balance del prestatario.    |
| `exit`     | Solicita salida diferida.                              | Shares bloqueadas y ticket emitido.   |

El objeto raíz incluye `scenario`, `network_id`, metadatos de activos, `vault`, `liquidity_stress`, `balances`, `surface`, `transactions`, `journal_entries`, `state_digest` y `conservation_ok`. Los digests son hexadecimales de 32 bytes. El campo `liquidity_stress` contiene `moderate` (1.000 bps) y `severe` (2.500 bps), cada uno con reserva inicial, pérdida, reserva estresada, mínimo, fees, colchón y cobertura del mínimo.

Los importes se serializan como números JSON. Un cliente JavaScript debe validar que caben en `Number.MAX_SAFE_INTEGER` antes de convertirlos a `number`; para el rango completo de `u128` se necesita un parser que preserve enteros o una capa de transporte que los represente como texto. Un consumidor debe fijar una versión y aceptar campos nuevos sin depender del orden de propiedades.

El escenario desconocido actualmente utiliza el flujo `routed` por defecto. No utilice ese comportamiento como mecanismo de descubrimiento; pase un nombre admitido explícitamente y compruebe `report.scenario` en la respuesta.

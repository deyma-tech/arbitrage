# TODO — ejecución y economía del bot

Documento de seguimiento para adaptar el bot a Chainstack y preparar una ejecución segura. Mantener `enable_execution = false` hasta cerrar los puntos de seguridad y validar un canary controlado.

## P0 — corregir antes de enviar fondos

- [ ] **Recalcular el costo real después de fijar el límite final de compute units.** Hoy los providers calculan un `TipResult`, luego pueden modificar `compute_unit_limit`, pero el `compute_unit_price` y `total_tip` pueden quedar calculados con el límite anterior. Recalcular inmediatamente antes de simular/enviar:
  - `priority_fee = ceil(compute_unit_price_micro_lamports * final_cu_limit / 1_000_000)`;
  - costo total conocido: priority fee + tip del provider + fee base + cualquier otro costo;
  - `net = gross_profit - costo_total`;
  - aplicar `min_net_profit_lamports` con esos valores finales.
- [ ] Mantener separados los dos límites económicos: `min_profit_lamports` como piso bruto y `min_net_profit_lamports` como piso después de costos.
- [ ] Verificar que la simulación tenga `err = null`, consuma las cuentas/estado esperados y que el dry-run nunca llegue a una operación de envío.
- [ ] Agregar métricas estructuradas por oportunidad: gross, tip, precio prioritario, CU solicitadas, CU consumidas, costo total, net, resultado de simulación, firma, slot y estado de confirmación.

## P1 — priority fees y compute units

- [ ] **Conectar el oracle de priority fees al camino activo.** `arb-bot/src/priority_fee.rs` consulta fees recientes, pero revisar que el valor elegido llegue efectivamente a Jito/Bloxroute/Nextblock/QuickNode en la configuración actual de Chainstack.
- [ ] Hacer configurable el criterio de selección: percentile (por ejemplo p75/p90), multiplicador, máximo y fallback cuando no haya datos. Registrar el valor elegido y su costo real.
- [ ] No copiar sin medir los valores del repo externo (`550000` CU y `50000` micro-lamports/CU). El costo depende del límite solicitado, y Solana cobra la prioridad usando ese límite aunque la transacción consuma menos.
- [ ] Reemplazar buffers fijos por calibración por ruta/DEX: simular, tomar `units_consumed`, sumar un margen configurable, aplicar un cap seguro y no superar el máximo de 1.4M CU. No reutilizar un límite histórico sin volver a pasar los guards.
- [ ] Medir antes/después: latencia hasta envío, porcentaje de simulaciones válidas, landing confirmado, costo prioritario y neto realizado.

## P1 — nonce accounts y envío

- [ ] Implementar un nonce manager explícito para las cuentas ya disponibles:
  - reservar una nonce account por lane concurrente;
  - no reutilizar la misma nonce simultáneamente;
  - incluir `AdvanceNonceAccount` como primera instrucción;
  - manejar refresh, reintentos y resultados ambiguos;
  - verificar inclusión on-chain, no solo aceptación del endpoint.
- [ ] Validar con canary que el nonce mejora la ventana de preparación/entrega. Un durable nonce evita depender de la expiración del recent blockhash, pero no garantiza por sí mismo menor latencia y agrega estado writable/posible contención.
- [ ] Diseñar fan-out de relayers con los mismos bytes firmados, deduplicación y tracking por firma. Varios relayers mejoran diversidad de entrega; no garantizan inclusión en el mismo bloque.
- [ ] Mantener claro el límite de responsabilidades: Chainstack aporta RPC/streams; no equivale automáticamente a un relayer de landing.

## P2 — cobertura de datos

- [ ] Mantener los dos streams de Chainstack para PumpSwap y Meteora DLMM.
- [ ] Si se agrega Raydium u otro DEX, evaluar primero el límite de streams/filters y el costo de suscripción. No ampliar el universo sin una estrategia de filtros acotados, snapshot o reparación en background.
- [ ] Preservar el estado en RAM acotado y controlar memoria/CPU antes de aumentar cobertura; `data/` debe seguir siendo runtime local e ignorado por git.

## P2 — checklist de canary

- [ ] Resolver el build completo (`protoc`/`nextblock-protos`) y ejecutar tests/checks.
- [ ] Confirmar ABI, cuentas y programa del executor V2 on-chain mediante simulación.
- [ ] Usar únicamente WSOL de la wallet configurada; flashloan permanece deshabilitado.
- [ ] Verificar ATA WSOL, balance, blockhash/nonce fresco y estado de pools antes de preparar la transacción.
- [ ] Activar ejecución solo explícitamente, con monto pequeño y límites bruto/neto altos; verificar firma, slot y `confirmationStatus` finalizado.
- [ ] Guardar evidencia del canary y reconciliar balance antes/después.

## Criterio de aceptación

El bot no se considera listo para operar con fondos hasta que el costo final se calcule con el CU limit definitivo, el neto pase el guard, la simulación sea válida y exista evidencia de inclusión on-chain y reconciliación de balance.

## Referencias revisadas

- Repo externo de referencia: <https://github.com/cutupdev/Solana-Arbitrage-Bot>
- Durable nonce oficial de Solana: <https://solana.com/developers/cookbook/transactions/durable-nonces>
- Estructura oficial de fees: <https://solana.com/docs/core/fees/fee-structure>
- Compute budget oficial de Solana: <https://solana.com/docs/core/fees/compute-budget>

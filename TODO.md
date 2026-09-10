# TODO — ejecución y economía del bot

Documento de seguimiento para adaptar el bot a Chainstack y preparar una ejecución segura. Mantener `enable_execution = false` hasta cerrar los puntos de seguridad y validar un canary controlado.

## P0 — corregir antes de enviar fondos

- [x] **Recalcular el costo real después de fijar el límite final de compute units.** El camino NextBlock recalcula priority fee, costo total y neto después de la calibración por simulación:
  - `priority_fee = ceil(compute_unit_price_micro_lamports * final_cu_limit / 1_000_000)`;
  - costo total conocido: priority fee + tip del provider + fee base + cualquier otro costo;
  - `net = gross_profit - costo_total`;
  - aplicar `min_net_profit_lamports` con esos valores finales.
- [ ] Mantener separados los dos límites económicos: `min_profit_lamports` como piso bruto y `min_net_profit_lamports` como piso después de costos.
- [ ] Verificar que la simulación tenga `err = null`, consuma las cuentas/estado esperados y que el dry-run nunca llegue a una operación de envío.
- [x] Agregar logs de dry-run para distinguir candidato, rechazo antes de simulación, preparación, simulación OK/rechazada/error RPC, CU consumidas y tiempo.
- [ ] Completar métricas estructuradas por oportunidad: gross, tip, precio prioritario, CU solicitadas, CU consumidas, costo total, net, resultado de simulación, firma, slot y estado de confirmación.

## P1 — priority fees y compute units

- [x] **Conectar el camino NextBlock y sus fees.** `NEXTBLOCK_API_KEY` alimenta el header Authorization, el endpoint gRPC usa TLS regional y el camino activo aplica CU price, tip y recálculo de costo final.
- [ ] Conectar el oracle de priority fees de `arb-bot/src/priority_fee.rs` al camino activo y validar su costo real.
- [ ] Hacer configurable el criterio de selección: percentile (por ejemplo p75/p90), multiplicador, máximo y fallback cuando no haya datos. Registrar el valor elegido y su costo real.
- [ ] No copiar sin medir los valores del repo externo (`550000` CU y `50000` micro-lamports/CU). El costo depende del límite solicitado, y Solana cobra la prioridad usando ese límite aunque la transacción consuma menos.
- [ ] Reemplazar buffers fijos por calibración por ruta/DEX: simular, tomar `units_consumed`, sumar un margen configurable, aplicar un cap seguro y no superar el máximo de 1.4M CU. No reutilizar un límite histórico sin volver a pasar los guards.
- [ ] Medir antes/después: latencia hasta envío, porcentaje de simulaciones válidas, landing confirmado, costo prioritario y neto realizado.

## P1 — nonce accounts y envío

- [x] Conectar las cuentas nonce configuradas al camino NextBlock y dry-run:
  - seleccionar cuentas en round-robin;
  - incluir `AdvanceNonceAccount` como primera instrucción;
  - reutilizar el mismo valor nonce entre simulación y envío;
  - limitar los lanes NextBlock al número de cuentas configuradas.
- [~] Nonce manager: ya reserva una cuenta por lane, comparte el nonce entre simulación/envío y espera `finalized`; ante error/timeout ambiguo la cuenta queda en cuarentena durante el proceso. Falta persistir/reconciliar cuarentenas después de un reinicio y definir una política de reintento.
- [ ] Validar con canary que el nonce mejora la ventana de preparación/entrega. Un durable nonce evita depender de la expiración del recent blockhash, pero no garantiza por sí mismo menor latencia y agrega estado writable/posible contención.
- [x] Integrar fan-out opcional a Jito, NextBlock, Astralane y Nozomi, con tips por proveedor y tracking local de firma/confirmación. Cada relay recibe una transacción firmada válida para su ruta; no se asume que todos compartan los mismos bytes ni que garanticen inclusión en el mismo bloque.
- [x] Aplicar los mínimos conocidos de tip: 0.001 SOL para Astralane y Nozomi; limitar Astralane a 5 envíos single-transaction por segundo y no usar su método bundle del tier free.
- [ ] Mantener claro el límite de responsabilidades: Chainstack aporta RPC/streams; no equivale automáticamente a un relayer de landing.

## P2 — cobertura de datos

- [x] Mantener dos filtros de cuentas en un stream Chainstack Yellowstone para PumpSwap y Meteora DLMM; el WebSocket estándar queda como fallback.
- [x] Deshabilitar por configuración las rutas de tres patas (`c3 = false`) y DLMM↔DLMM (`allow_dlmm_dlmm = false`) en el perfil dry-run, preservando el código para futuras extensiones.
- [ ] Si se agrega Raydium u otro DEX, evaluar primero el límite de streams/filters y el costo de suscripción. No ampliar el universo sin una estrategia de filtros acotados, snapshot o reparación en background.
- [ ] Preservar el estado en RAM acotado y controlar memoria/CPU antes de aumentar cobertura; `data/` debe seguir siendo runtime local e ignorado por git.
- [x] Implementar un snapshot de mercado separado del almacenamiento de ALT en `data/market-snapshot.bin`, con versión, slot de referencia y escritura atómica (`tmp` + rename). Se guardan bytes crudos y se reconstruyen con los decoders existentes; no se serializa ciegamente todo el `GPAResult`.
- [x] Guardar en el snapshot el universo admitido por el perfil activo y sus dependencias exactas: pools WSOL PumpSwap/DLMM, `BinArray` no vacíos y bitmap extensions. El formato queda preparado para ampliar token accounts/reservas si el perfil las requiere.
- [x] En el arranque, cargar el snapshot mientras se conecta Yellowstone y solicitar replay desde el slot siguiente, preservando el descubrimiento incremental de cuentas nuevas y cambios recientes.
- [ ] Para cada pool nuevo, hidratar también sus dependencias antes de incorporarlo al cálculo: reservas/supply/token accounts en PumpSwap y `BinArray`/bitmap en Meteora DLMM. No habilitar ejecución con una ruta parcialmente hidratada.
- [ ] Agregar reconciliación después de cada reconexión de Yellowstone y periódicamente: detectar cuentas creadas, modificadas o eliminadas durante el hueco del stream. El replay desde snapshot cubre el arranque, pero no reemplaza esta reconciliación histórica.
- [ ] Registrar métricas de cold start y reconciliación: edad/slot del snapshot, cuentas reutilizadas, nuevas, eliminadas, reparadas, dependencias faltantes y tiempo hasta el primer estado quoteable.

## P2 — checklist de canary

- [x] Resolver el build completo (`protoc`/`nextblock-protos`) y ejecutar tests/checks. `cargo check -p arb-bot`, tests unitarios de `arb-core` y release build pasan; el doctest del crate local `config` sigue chocando con la dependencia externa homónima `config@0.15.11`.
- [ ] Confirmar ABI, cuentas y programa del executor V2 on-chain mediante una simulación de canary con los providers elegidos.
- [ ] Usar únicamente WSOL de la wallet configurada; flashloan permanece deshabilitado.
- [x] Verificar en el camino de preparación el ATA WSOL configurado, usar sólo WSOL de la wallet, leer un nonce fresco y aplicar los guards de estado/profit antes de preparar la transacción.
- [ ] Ejecutar el canary real: usar `enable_execution = true`, `providers = "jito,astralane,nozomi,nextblock"`, monto pequeño, una sola oportunidad admitida y esperar `finalized`; no considerar HTTP 200/aceptación del relay como éxito.
- [ ] Activar ejecución solo explícitamente, con monto pequeño y límites bruto/neto altos; verificar firma, slot y `confirmationStatus` finalizado.
- [ ] Guardar evidencia del canary y reconciliar balance antes/después.

## Criterio de aceptación

El bot no se considera listo para operar con fondos hasta que el costo final se calcule con el CU limit definitivo, el neto pase el guard, la simulación sea válida y exista evidencia de inclusión on-chain y reconciliación de balance.

## Referencias revisadas

- Repo externo de referencia: <https://github.com/cutupdev/Solana-Arbitrage-Bot>
- Durable nonce oficial de Solana: <https://solana.com/developers/cookbook/transactions/durable-nonces>
- Estructura oficial de fees: <https://solana.com/docs/core/fees/fee-structure>
- Compute budget oficial de Solana: <https://solana.com/docs/core/fees/compute-budget>
- Suscripción oficial `programSubscribe`: <https://solana.com/docs/rpc/websocket/programsubscribe>

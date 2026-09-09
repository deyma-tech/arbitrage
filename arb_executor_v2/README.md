# `arb_executor_v2`

Copia local del programa on-chain `arb_executor_v2` que se usa con el
Program ID:

`HPXVR7EQc1KE9XAY4wPoTakBTFgs4SNZW75zBaVfufW3`

Este paquete permanece separado del bot y del executor genérico legacy.
Acepta únicamente los tags V2 `arbv2d02` y `arbv2p02`, con rutas
PumpSwap <-> Meteora DLMM y guard atómico de profit.

## Build

```bash
cargo check -p arb-executor-v2
```

El bot debe usar `ARB_EXECUTOR_V2_PROGRAM_ID` para esta ruta. No debe
reemplazarse por `SWAP_PROGRAM_ID`: ese nombre corresponde al ABI genérico
legacy y no es compatible con este programa.

No se despliega automáticamente desde este directorio.

# Rust S24 Evidence Bundle & CI/Local Parity — diseño

Fecha: 2026-09-15

## Objetivo

Convertir la evidencia del gate productivo en un paquete machine-readable,
sanitizado y comparable entre GitHub Actions y la ejecución local mediante
WSL2, conservando TCP como decisión principal y `100000` frames como meta
obligatoria.

## Problema

S20/S21 ya ejecutan el gate Linux en GitHub Actions y S23 deja preparado el
launcher WSL2, pero la evidencia se valida principalmente por documentos y
archivos producidos por cada flujo. Falta un contrato único que permita
determinar si un paquete está completo, si su resultado es comparable y si fue
sanitizado, sin confundir `NO EJECUTADA` con `FAIL`.

## Diseño

S24 añadirá un esquema JSON versionado para un bundle de evidencia y un
validador independiente reutilizable por CI y por el launcher local. El bundle
contendrá metadatos de ejecución, preflight, casos de smoke, gate comparable y
lifecycle. Cada caso registrará protocolo, frames, batch, estado, duración,
usuario efectivo, digest, commit y motivo cuando no se ejecute.

El validador aplicará reglas deterministas:

- `PASS` TCP comparable exige exactamente `frames_sent=100000` y
  `frames_ok=100000`.
- `PASS` smoke exige `frames_sent=3`, `frames_ok=3` y
  `synthetic_cycle=false`.
- `NO EJECUTADA` exige un motivo no vacío y no puede presentarse como
  aceptación del gate.
- `FAIL` exige código de salida o diagnóstico observable.
- Todo bundle debe declarar protocolo, versión de esquema, commit y digest.
- No se permiten PEM, tokens, credenciales ni valores de variables secretas.

HTTP `100000` seguirá siendo medición adicional y nunca sustituirá la decisión
TCP. El bundle no almacenará payloads, secretos ni contenido sensible; solo
metadatos y referencias sanitizadas a artefactos.

## Integración

El workflow `production-readiness.yml` generará y validará un bundle CI antes
de cargarlo como artifact. El launcher S23 podrá validar el bundle local al
finalizar, preservando su código de salida. La documentación explicará cómo
comparar bundles por commit, protocolo, frames, digest y estado.

## Fuera de alcance

- Instalar WSL2, una distro Linux o Docker Desktop.
- Cambiar el runtime de telemetría, TCP default o el flujo HTTP.
- Cambiar la meta comparable de `100000` frames.
- Declarar aceptación local mientras la distro Linux siga ausente.
- Añadir benchmarks nuevos o sustituir la evidencia de GitHub Actions.

## Aceptación

S24 queda aceptado cuando un bundle CI y uno WSL2 válido pasan el mismo
validador, el gate TCP solo acepta `100000/100000`, los estados ambientales se
representan como `NO EJECUTADA`, el escaneo de secretos pasa y el workflow
publica evidencia sanitizada reproducible.

# Rust S22 — Local Linux Runner & Reproducible Production Gate

## Goal

Cerrar la brecha operacional pendiente permitiendo ejecutar localmente el mismo
gate Docker/TLS/non-root/lifecycle de S20 y S21 desde PowerShell, usando WSL2 o
un entorno Linux existente, sin cambiar el criterio comparable de `100000`
frames.

## Context

S20 validó el runtime productivo en Ubuntu mediante GitHub Actions y S21 añadió
el flujo HTTP/TLS. El repositorio ya contiene la lógica principal en
`scripts/docker_production_readiness.sh`; S22 debe exponer esa capacidad
localmente sin duplicar el gate ni reclamar soporte de un entorno que no esté
instalado.

## Scope

S22 incluye:

- launcher PowerShell para ejecutar el gate local en WSL2/Linux;
- detección explícita de WSL, Bash y Docker antes de iniciar contenedores;
- propagación de `PRISM_PROTOCOL`, `--frames`, `--batch-size` y directorio de
  evidencia;
- defaults de producción con `100000` frames y `PRISM_ALLOW_CYCLE=false`;
- smoke reducido opcional para validación rápida;
- evidencia local sanitizada con marcador de entorno y resultado del gate;
- documentación de prerrequisitos, ejecución, diagnóstico y limpieza;
- contratos automatizados del launcher y regresiones del gate existente.

S22 no incluye:

- instalación automática de WSL2, Docker Desktop o dependencias del sistema;
- reemplazo o modificación del workflow de GitHub Actions;
- un nuevo servidor, protocolo, endpoint o cliente;
- sustitución de `100000` por otra meta de decisión;
- ejecución de un dataset externo de 1,000,000 de archivos.

## Architecture

El launcher será una frontera de operador, no una segunda implementación del
runtime. Validará el entorno local y traducirá argumentos PowerShell a la
invocación Linux del script existente:

```text
PowerShell launcher
    -> prerequisite checks
    -> WSL/Linux bash
    -> docker_production_readiness.sh
    -> Compose server/client/telemetry-client
    -> sanitized evidence
```

La lógica de TLS efímero, secretos, Compose, non-root, smoke, carga comparable,
lifecycle, reconnect y limpieza seguirá siendo responsabilidad del script
existente. El launcher debe conservar el código de salida y mostrar la ruta de
evidencia para facilitar auditoría.

## Interface

El comando será:

```powershell
.\scripts\run_production_gate.ps1 [-Frames 100000] [-BatchSize 64] `
  [-Protocol tcp|http] [-EvidenceDir <path>] [-LinuxDistribution <name>]
```

Reglas:

- `Frames` debe ser entero positivo; por defecto `100000`.
- `BatchSize` debe ser entero positivo; por defecto `64`.
- `Protocol` solo acepta `tcp` o `http`; por defecto `tcp`.
- `EvidenceDir` debe resolverse dentro del repositorio o ser una ruta absoluta
  explícita; no se imprimirán secretos.
- si no existe WSL, Bash o Docker accesible desde Linux, el comando termina con
  código distinto de cero y un mensaje que identifica el prerrequisito faltante;
  no instala ni modifica el sistema.
- la distribución se detecta automáticamente cuando no se indica;
  `LinuxDistribution` solo selecciona una distribución ya instalada.

## Security and evidence

- Los secretos continúan siendo efímeros y se generan dentro del gate existente.
- No se copiarán PEM, tokens ni variables secretas a la evidencia.
- Los artefactos locales deben incluir solo resultados, marcadores, hashes y
  metadatos no sensibles.
- Se conservará `PRISM_CONTAINER_USER=10001:10001`.
- El launcher no aceptará secretos como argumentos de línea de comandos.

## Verification

Los contratos deben cubrir defaults, validación de argumentos, selección de
WSL/distribución, propagación de variables y errores accionables. La verificación
local debe incluir:

1. sintaxis PowerShell y Bash;
2. contratos Python existentes y nuevos;
3. `docker compose config` base y producción;
4. smoke TCP y HTTP cuando el entorno local esté disponible;
5. gate completo de `100000` frames cuando Docker/WSL estén disponibles;
6. suite Python, suite Cargo release, `git diff --check` y escaneo de secretos.

La ejecución real del gate local se registrará como PASS, FAIL o NO EJECUTADA
según la disponibilidad del entorno. No se declarará soporte Linux local por
el mero hecho de que el launcher exista.

## Acceptance criteria

- Un operador puede ejecutar el gate con un comando PowerShell documentado.
- Los errores por prerrequisitos faltantes son deterministas y accionables.
- TCP sigue siendo el default y HTTP puede seleccionarse explícitamente.
- `100000` permanece como meta comparable para la carga productiva.
- El launcher no duplica la lógica de producción ni debilita CI.
- La evidencia local es sanitizada y reproducible.
- La suite completa permanece verde.

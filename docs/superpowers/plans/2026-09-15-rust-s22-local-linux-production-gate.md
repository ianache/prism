# Rust S22 Local Linux Production Gate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Ejecutar localmente el mismo gate Docker/TLS/non-root/lifecycle de S20/S21 desde PowerShell mediante WSL2 o Linux existente, conservando `100000` como meta comparable.

**Architecture:** Un launcher PowerShell validará WSL, Bash y Docker, resolverá la distribución Linux y traducirá los argumentos a `scripts/docker_production_readiness.sh`. El launcher no duplicará la lógica de TLS, secretos, Compose, carga ni limpieza; solo propagará configuración, código de salida y ruta de evidencia.

**Tech Stack:** PowerShell 5.1+/7, WSL2, Bash, Docker Compose, Python estándar para contratos, pytest y Cargo release.

**Spec:** `docs/superpowers/specs/2026-09-15-rust-s22-local-linux-production-gate-design.md`

## Global Constraints

- TCP continúa siendo el protocolo por defecto; HTTP se selecciona con `-Protocol http`.
- `100000` permanece como meta obligatoria de decisión para gates comparables.
- No instalar automáticamente WSL2, Docker Desktop ni dependencias del sistema.
- No modificar la lógica interna del gate existente salvo para habilitar una invocación local segura.
- No aceptar secretos como argumentos del launcher.
- Mantener `PRISM_ALLOW_CYCLE=false` y `PRISM_CONTAINER_USER=10001:10001`.
- No incluir PEM, tokens ni rutas secretas en logs o evidencia.
- Mantener `graphify-out/` y archivos temporales del usuario fuera de commits.
- Cada tarea debe empezar con una prueba o contrato que falle y terminar con su verificación y commit.

---

### Task 1: Definir el contrato del launcher PowerShell

**Files:**
- Create: `scripts/run_production_gate.ps1`
- Create: `tests/test_s22_local_runner_contract.py`

**Interfaces:**
- Consumes: parámetros PowerShell `Frames`, `BatchSize`, `Protocol`, `EvidenceDir`, `LinuxDistribution`.
- Produces: contrato estable para invocar `scripts/docker_production_readiness.sh` y mensajes de prerrequisitos.

- [ ] Escribir contratos fallidos que exijan el archivo, parámetros por defecto `Frames=100000`, `BatchSize=64`, `Protocol=tcp`, `PRISM_ALLOW_CYCLE=false` y la referencia al script de readiness.
- [ ] Añadir contratos fallidos para aceptar solo `tcp|http`, rechazar frames/batch cero o negativos y rechazar argumentos que intenten transportar tokens o PEM.
- [ ] Añadir contratos fallidos para mensajes deterministas cuando falte WSL, Bash, Docker o una distribución solicitada.
- [ ] Ejecutar `python -m pytest -q tests/test_s22_local_runner_contract.py` y confirmar que falla porque el launcher aún no existe.
- [ ] Definir en el launcher un `param` block con tipos explícitos, valores por defecto y `ValidateSet('tcp','http')` para `Protocol`.
- [ ] Implementar una función `Write-Failure([string]$Message)` que escriba en stderr y termine con código 2 para errores de argumentos/prerrequisitos.
- [ ] Implementar la resolución de distribución: usar `-LinuxDistribution` si fue indicada; de lo contrario usar la primera distribución devuelta por `wsl.exe -l -q`.
- [ ] Ejecutar los contratos enfocados y verificar PASS.
- [ ] Commit: `feat: add local production gate launcher contract`.

### Task 2: Implementar la invocación WSL/Linux segura

**Files:**
- Modify: `scripts/run_production_gate.ps1`
- Modify: `tests/test_s22_local_runner_contract.py`

**Interfaces:**
- Consumes: resolución de Task 1 y parámetros validados.
- Produces: invocación de `bash scripts/docker_production_readiness.sh` con `--frames`, `--batch-size` y `--evidence-dir`, preservando código de salida.

- [ ] Añadir un contrato fallido que compruebe que el launcher ejecuta `wsl.exe --distribution <distro> -- bash -lc ...` y no un shell Windows para el script Bash.
- [ ] Añadir un contrato fallido que compruebe la propagación literal de `PRISM_PROTOCOL`, `--frames`, `--batch-size` y `--evidence-dir`.
- [ ] Añadir un contrato fallido que exija que el directorio de evidencia se cree/resuelva sin interpolar secretos en la línea de comandos.
- [ ] Implementar `Test-CommandAvailable` para comprobar `wsl.exe` y `docker.exe`, y dentro de WSL comprobar `bash` y `docker compose version`.
- [ ] Resolver la raíz del repositorio con `git rev-parse --show-toplevel` y convertirla a una ruta `/mnt/<drive>/...` para WSL mediante `wsl.exe wslpath -a -u`.
- [ ] Ejecutar el gate con una lista de argumentos, evitando concatenar valores no confiables en una orden Bash; exportar solo `PRISM_PROTOCOL` y usar los flags existentes del script.
- [ ] Crear el directorio de evidencia antes de la ejecución y mostrar únicamente su ruta y el código de salida.
- [ ] Devolver exactamente el código de salida del gate; usar código 2 solo para validación/prerrequisitos.
- [ ] Ejecutar contratos enfocados y una prueba manual simulada con un stub de `wsl.exe`; confirmar PASS.
- [ ] Commit: `feat: invoke production gate through WSL`.

### Task 3: Añadir matriz local y evidencia sanitizada

**Files:**
- Create: `tests/test_s22_local_runner_matrix.py`
- Modify: `scripts/run_production_gate.ps1` si se requiere un selector de modo
- Create: `docs/evidence/rust-s22-local-linux-production-gate-2026-09-15.md`

**Interfaces:**
- Consumes: launcher funcional y gate S18/S20/S21.
- Produces: matriz documentada para TCP/HTTP y smoke/comparable, con estados PASS, FAIL o NO EJECUTADA.

- [ ] Escribir contratos fallidos para que la matriz documente exactamente cuatro combinaciones: `tcp smoke`, `http smoke`, `tcp 100000` y `http 100000`.
- [ ] Exigir que el contrato no marque PASS cuando el entorno no está disponible y que registre `NO EJECUTADA` con el prerrequisito faltante.
- [ ] Implementar la matriz como contrato/documentación, sin crear un segundo runner ni cambiar el gate Bash.
- [ ] Ejecutar el launcher con `-Frames 3 -BatchSize 1 -Protocol tcp` y `-Protocol http` cuando WSL/Docker estén disponibles; revisar `frames_ok=3` y `synthetic_cycle=false`.
- [ ] Ejecutar el gate completo con `-Frames 100000 -BatchSize 64 -Protocol tcp` cuando el entorno local esté disponible; no sustituirlo por una corrida reducida para aceptación.
- [ ] Ejecutar la variante HTTP de `100000` solo como medición adicional y documentar que no reemplaza el gate comparable TCP si el tiempo/capacidad local no permite completarla.
- [ ] Auditar evidencia con `rg` para confirmar ausencia de PEM, tokens, `PRISM_AUTH_TOKEN` y secretos efímeros.
- [ ] Escribir la evidencia con host, distribución, versiones, comandos abstractos, estados y limitaciones, sin valores sensibles.
- [ ] Commit: `test: add S22 local gate matrix and evidence`.

### Task 4: Documentar operación y diagnóstico

**Files:**
- Modify: `docs/guia-servidor-supervisor-cliente-tls.md`
- Modify: `.env.example` solo si el launcher necesita una variable no existente
- Modify: `tests/test_s22_local_runner_contract.py`

**Interfaces:**
- Consumes: interfaz final del launcher y resultados de la matriz.
- Produces: guía PowerShell reproducible para prerrequisitos, smoke, gate `100000`, HTTP/TCP, errores y limpieza.

- [ ] Añadir contrato fallido para que la guía contenga el comando base, `-Frames 3`, `-Protocol http`, `-Protocol tcp`, `-EvidenceDir` y el gate de `100000`.
- [ ] Documentar comprobaciones manuales de `wsl --status`, `wsl -l -v`, `docker version` y `docker compose version` sin instalarlas automáticamente.
- [ ] Documentar la diferencia entre `NO EJECUTADA` por entorno ausente y `FAIL` por fallo del gate.
- [ ] Documentar limpieza del directorio de evidencia y del stack Docker sin borrar rutas amplias ni secretos del usuario.
- [ ] Documentar que GitHub Actions sigue siendo la puerta Linux ya validada cuando el runner local no está disponible.
- [ ] Ejecutar los contratos y revisar que la documentación no contradiga la meta `100000` ni el default TCP.
- [ ] Commit: `docs: document local Linux production gate`.

### Task 5: Verificación completa y cierre auditable

**Files:**
- Modify: `docs/consumo.md`
- Modify: `docs/evidence/rust-s22-local-linux-production-gate-2026-09-15.md`

**Interfaces:**
- Consumes: Tasks 1–4, suite existente y disponibilidad real del entorno local.
- Produces: evidencia final, consumo registrado y rama lista para revisión/integración.

- [ ] Ejecutar `python -m pytest -q` y guardar el resultado exacto.
- [ ] Ejecutar `cargo test --workspace --release` y guardar el resultado exacto.
- [ ] Ejecutar contratos S22, `docker compose config` base/producción, `git diff --check` y el escaneo dirigido de secretos.
- [ ] Confirmar que las aserciones de benchmark/gate conservan exactamente `100000`.
- [ ] Registrar en `docs/consumo.md` una fila por tarea S22 y una fila final del plan, usando `N/D` cuando el proveedor no exponga tokens o tiempos precisos.
- [ ] Actualizar la evidencia con commit final, resultados locales, estado de la matriz y la limitación real del entorno.
- [ ] Revisar el plan y marcar sus pasos completados únicamente después de la verificación correspondiente.
- [ ] Commit: `docs: finalize S22 local Linux production gate evidence`.

## Verification Matrix

| Área | Evidencia requerida |
|---|---|
| Launcher | Defaults, validación, distribución y códigos de salida |
| Prerrequisitos | WSL/Bash/Docker detectados sin instalación automática |
| Runtime | Smoke TCP/HTTP y gate comparable `100000` cuando el entorno esté disponible |
| Seguridad | Sin secretos en argumentos, logs ni evidencia; non-root preservado |
| Compatibilidad | Workflow CI y script Bash existentes sin debilitamiento |
| Documentación | Comandos PowerShell, diagnóstico, limpieza y estados NO EJECUTADA/FAIL |
| Auditoría | Suite Python/Cargo, Compose config, diff check y consumo registrado |

## Completion Gate

S22 se completa cuando el launcher funciona o reporta de forma determinista la
ausencia de prerrequisitos, los contratos y suites pasan, la matriz queda
documentada, la evidencia es sanitizada y `docs/consumo.md` está actualizado.
La aceptación comparable sigue requiriendo exactamente `100000` frames; un
smoke reducido no puede reemplazarla. Si el entorno Linux local no está
disponible, debe quedar explícitamente como `NO EJECUTADA`, mientras el gate de
GitHub Actions permanece como evidencia operacional válida.

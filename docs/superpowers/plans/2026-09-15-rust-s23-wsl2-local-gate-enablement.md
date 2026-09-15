# Rust S23 WSL2 Local Gate Enablement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Habilitar y verificar la ejecución local real del gate S22 desde WSL2 con Docker Desktop, conservando exactamente `100000` como meta comparable.

**Architecture:** La preparación del host permanece manual y fuera del repositorio. El código validará prerrequisitos y el launcher S22 ejecutará el script Bash existente; S23 añadirá contratos, documentación, diagnóstico y evidencia de la ejecución real sin duplicar el runtime.

**Tech Stack:** WSL2, distribución Linux de usuario, Docker Desktop, Docker Compose, PowerShell, Bash, pytest y Cargo release.

**Spec:** `docs/superpowers/specs/2026-09-15-rust-s23-wsl2-local-gate-enablement-design.md`

## Global Constraints

- No instalar automáticamente WSL2, distribuciones Linux ni Docker Desktop.
- `docker-desktop` y `docker-desktop-data` no son distribuciones operativas válidas.
- TCP sigue siendo el default y HTTP se selecciona explícitamente.
- La meta comparable es exactamente `100000` frames.
- Un smoke reducido no sustituye el gate comparable.
- Mantener `PRISM_ALLOW_CYCLE=false` y `PRISM_CONTAINER_USER=10001:10001`.
- No exponer PEM, tokens, credenciales ni variables secretas en argumentos, logs o evidencia.
- No debilitar el workflow CI ni el gate Bash de S22.
- Cada tarea comienza con una prueba/contrato fallido y termina con verificación y commit.

---

### Task 1: Contrato de prerrequisitos WSL2 y Docker

**Files:**
- Create: `tests/test_s23_wsl2_contract.py`
- Modify: `scripts/run_production_gate.ps1` solo si el contrato revela una brecha

**Interfaces:**
- Consumes: launcher S22 y comandos de diagnóstico del host.
- Produces: contrato estable para distinguir WSL ausente, distro interna, distro de usuario y Docker no accesible.

- [ ] Escribir contratos fallidos que exijan en la guía y launcher las comprobaciones `wsl.exe --status`, `wsl.exe -l -v`, `docker.exe version` y `docker.exe compose version`.
- [ ] Exigir que el launcher excluya `docker-desktop` y `docker-desktop-data` y emita `NO Linux distribution is installed` cuando no haya otra distro.
- [ ] Exigir que el contrato documente los comandos Linux `docker version`, `docker compose version` y `bash --version`.
- [ ] Ejecutar `python -m pytest -q tests/test_s23_wsl2_contract.py` y confirmar fallo por ausencia del contrato S23.
- [ ] Implementar únicamente las aserciones/documentación faltantes; no agregar instalación automática.
- [ ] Ejecutar los contratos S23 y los contratos S22 del launcher; confirmar PASS.
- [ ] Commit: `test: add S23 WSL2 prerequisite contracts`.

### Task 2: Documentar preparación manual del entorno

**Files:**
- Modify: `docs/guia-servidor-supervisor-cliente-tls.md`
- Modify: `tests/test_s23_wsl2_contract.py`

**Interfaces:**
- Consumes: contrato de prerrequisitos de Task 1.
- Produces: procedimiento reproducible para que un operador prepare una distro Linux de usuario y la integre con Docker Desktop.

- [ ] Añadir un contrato fallido para instrucciones de instalación manual de una distro Linux desde Microsoft Store o `wsl --install -d <Distro>`, dejando claro que el operador ejecuta la instalación.
- [ ] Documentar la comprobación de virtualización, WSL2, versión de distro (`VERSION 2`) y Docker Desktop con integración WSL habilitada.
- [ ] Documentar la comprobación desde Linux y la ubicación recomendada del repositorio bajo `/mnt/d/...` sin copiar secretos al filesystem Linux.
- [ ] Documentar los errores de daemon no accesible, permisos Docker, rutas Windows/WSL y distribución incorrecta.
- [ ] Documentar que no debe usarse `docker-desktop-data` como distro operativa.
- [ ] Añadir comandos S22 para smoke TCP, smoke HTTP y gate TCP `100000` con directorios de evidencia separados.
- [ ] Ejecutar contratos y revisar que los comandos no contradigan TCP default, TLS ni `100000`.
- [ ] Commit: `docs: document manual WSL2 gate preparation`.

### Task 3: Validar launcher contra una distro Linux real

**Files:**
- Modify: `scripts/run_production_gate.ps1` si se encuentran diferencias de WSL real
- Modify: `tests/test_s22_local_runner_contract.py`
- Create: `tests/test_s23_launcher_integration.py`

**Interfaces:**
- Consumes: launcher S22, distribución indicada por `-LinuxDistribution` y raíz del repositorio.
- Produces: ejecución comprobable del script Bash con rutas WSL, protocolo, frames, batch y código de salida preservados.

- [ ] Escribir una prueba fallida que use una distro Linux explícita simulada y verifique `wslpath`, `bash -lc`, `PRISM_PROTOCOL` y los flags de readiness.
- [ ] Escribir una prueba fallida que verifique que el launcher crea/resuelve `EvidenceDir` y no pasa secretos por argumentos.
- [ ] Ejecutar las pruebas antes de cambios y confirmar que la integración simulada no existe.
- [ ] Corregir solo las diferencias descubiertas: encoding de `wsl.exe`, quoting Bash, conversión de rutas o propagación de código de salida.
- [ ] Ejecutar el launcher real con `-LinuxDistribution <distro>` y `-Frames 3 -BatchSize 1 -Protocol tcp`; esperar `frames_ok=3` y `synthetic_cycle=false`.
- [ ] Repetir el smoke real con `-Protocol http` y evidencia separada.
- [ ] Ejecutar la prueba de integración simulada, los contratos S22/S23 y el parseo PowerShell.
- [ ] Commit: `feat: validate S22 launcher on WSL2`.

### Task 4: Ejecutar la matriz productiva y generar evidencia S23

**Files:**
- Create: `tests/test_s23_local_gate_evidence.py`
- Modify: `docs/evidence/rust-s22-local-linux-production-gate-2026-09-15.md`
- Create: `docs/evidence/rust-s23-wsl2-local-gate-enablement-2026-09-15.md`

**Interfaces:**
- Consumes: launcher validado, distro Linux real, Docker Desktop y gate S22.
- Produces: evidencia PASS/FAIL/NO EJECUTADA para smoke TCP/HTTP y carga comparable TCP.

- [ ] Escribir contratos fallidos para una matriz de cuatro casos: `tcp smoke`, `http smoke`, `tcp 100000` y `http 100000`.
- [ ] Exigir que cada caso registre comando abstracto, estado, duración, frames, usuario efectivo, digest y motivo si no se ejecutó.
- [ ] Ejecutar smoke TCP y HTTP con `3` frames, batch `1`, `synthetic_cycle=false` y evidencia separada.
- [ ] Ejecutar el gate comparable TCP con `100000` frames y batch `64`; aceptar solo `frames_sent=100000` y `frames_ok=100000`.
- [ ] Ejecutar HTTP `100000` únicamente como medición adicional, documentando que no reemplaza la decisión TCP.
- [ ] Ejecutar escaneo dirigido sobre la evidencia y confirmar que no contiene PEM, tokens, `PRISM_AUTH_TOKEN` ni credenciales.
- [ ] Registrar versiones de Windows/WSL/distro/Docker/Compose y el resultado real de cada caso.
- [ ] Si el host sigue sin distro válida, registrar `NO EJECUTADA` con el diagnóstico y conservar GitHub Actions como evidencia operativa.
- [ ] Commit: `test: capture S23 WSL2 local gate evidence`.

### Task 5: Verificación completa y cierre auditable

**Files:**
- Modify: `docs/consumo.md`
- Modify: `docs/evidence/rust-s23-wsl2-local-gate-enablement-2026-09-15.md`
- Modify: `docs/superpowers/plans/2026-09-15-rust-s23-wsl2-local-gate-enablement.md`

**Interfaces:**
- Consumes: Tasks 1–4, suites del repositorio y matriz local.
- Produces: plan marcado, consumo registrado y rama lista para revisión/integración.

- [ ] Ejecutar `python -m pytest -q` y registrar el conteo exacto.
- [ ] Ejecutar `cargo test --workspace --release` y registrar resultado PASS/FAIL.
- [ ] Ejecutar `docker compose config` base/producción, `git diff --check`, parseo PowerShell y escaneo de secretos.
- [ ] Confirmar que toda aserción de benchmark/gate conserva exactamente `100000`.
- [ ] Actualizar `docs/consumo.md` con una fila por tarea S23 y una fila final del plan, usando `N/D` para tokens/tiempos no expuestos.
- [ ] Completar evidencia con commit final, matriz real, limitaciones y diferencia entre CI y local.
- [ ] Marcar los pasos del plan como completados solo después de observar sus verificaciones.
- [ ] Commit: `docs: finalize S23 WSL2 local gate evidence`.

## Verification Matrix

| Área | Evidencia requerida |
|---|---|
| Host | WSL2, distro de usuario, Docker Desktop y Compose detectados |
| Launcher | Rutas WSL, quoting, encoding, parámetros y códigos de salida |
| Runtime | Smoke TCP/HTTP y gate TCP `100000` si el host está preparado |
| Seguridad | Secrets fuera de argumentos, logs y evidencia; non-root preservado |
| Compatibilidad | Gate Bash S22 y workflow CI sin cambios debilitantes |
| Documentación | Instalación manual, diagnóstico, limpieza y estados explícitos |
| Auditoría | Matriz, suites Python/Cargo, Compose config y consumo registrado |

## Completion Gate

S23 se completa cuando el operador puede preparar manualmente WSL2 y Docker,
el launcher ejecuta el gate real o registra un bloqueo ambiental determinista,
la matriz y evidencia están sanitizadas, las suites pasan y `docs/consumo.md`
está actualizado. La aceptación comparable exige exactamente `100000` frames;
los smoke no la sustituyen.

## Cierre de ejecución

- [x] Tarea 1: prerrequisitos WSL2 y Docker.
- [x] Tarea 2: preparación manual del entorno.
- [x] Tarea 3: validación del launcher WSL2.
- [x] Tarea 4: matriz y evidencia local sanitizada.
- [x] Tarea 5: verificación completa, consumo y cierre auditable.

Las tareas 1–5 fueron ejecutadas y verificadas. La matriz local conserva cuatro
casos `NO EJECUTADA` porque el host no tiene una distribución Linux de usuario;
esta condición no se presenta como PASS ni como FAIL del runtime. La suite
Python, Cargo release, Compose, parseo PowerShell, diff y escaneo de secretos
quedaron verificados. La meta comparable permanece exactamente `100000`.

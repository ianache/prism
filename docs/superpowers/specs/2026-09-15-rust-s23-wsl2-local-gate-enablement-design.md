# Rust S23 — WSL2 Local Gate Enablement

## Goal

Habilitar una ejecución local real del gate S22 mediante una distribución Linux
de usuario en WSL2 integrada con Docker Desktop, conservando el criterio
comparable de exactamente `100000` frames.

## Context

S22 implementó el launcher PowerShell y verificó sus contratos, pero el host
actual solo expone las distribuciones internas `docker-desktop` y
`docker-desktop-data`. Por eso la matriz local quedó como `NO EJECUTADA`. S23
documentará y validará la preparación manual del entorno, sin convertir el
repositorio en un instalador del sistema.

## Scope

S23 incluye:

- guía manual para instalar una distribución Linux en WSL2;
- validación de WSL2, versión de distribución y Docker Desktop;
- comprobación de integración Docker desde Linux con `docker version` y
  `docker compose version`;
- recomendaciones de ubicación del repositorio y montajes Windows/WSL;
- ejecución real de smoke TCP y HTTP mediante el launcher S22;
- ejecución real del gate comparable TCP con `100000` frames;
- evidencia S23 con distribución, versiones, estados y sanitización;
- diagnóstico de permisos, red, certificados, mounts y Docker daemon;
- contratos documentales que prevengan instrucciones inseguras o automáticas.

S23 no incluye:

- instalación automática de WSL2, una distribución Linux o Docker Desktop;
- modificación del kernel, políticas de seguridad o configuración global del host;
- sustitución de GitHub Actions como puerta Linux ya validada;
- cambio del launcher S22 salvo correcciones necesarias descubiertas por la
  ejecución real;
- cambio del protocolo TCP default, del flujo HTTP o de la meta `100000`;
- soporte de Kubernetes, máquinas virtuales adicionales o runners remotos.

## Architecture

El operador prepara el host fuera del repositorio y luego ejecuta el launcher
S22 desde PowerShell. La distribución Linux proporciona Bash y el acceso al
daemon Docker Desktop; el repositorio sigue siendo responsable únicamente del
gate y de la evidencia.

```text
Operador
  -> WSL2 + distro Linux de usuario
  -> integración Docker Desktop
  -> launcher S22 desde PowerShell
  -> docker_production_readiness.sh
  -> evidencia local sanitizada
```

La guía debe distinguir claramente tres estados: prerrequisito ausente
(`NO EJECUTADA`), entorno disponible con fallo del gate (`FAIL`) y ejecución
aceptada (`PASS`). La instalación es siempre una acción explícita del operador.

## Supported host contract

Antes de ejecutar el gate, el operador debe poder demostrar:

```powershell
wsl.exe --status
wsl.exe -l -v
docker.exe version
docker.exe compose version
```

Dentro de una distribución Linux de usuario, deben funcionar:

```bash
docker version
docker compose version
bash --version
```

Las distribuciones internas `docker-desktop` y `docker-desktop-data` no son
válidas como distribución operativa para el launcher.

## Execution contract

Smoke TCP:

```powershell
.\scripts\run_production_gate.ps1 -Frames 3 -BatchSize 1 -Protocol tcp `
  -EvidenceDir .\artifacts\s23-tcp-smoke
```

Smoke HTTP:

```powershell
.\scripts\run_production_gate.ps1 -Frames 3 -BatchSize 1 -Protocol http `
  -EvidenceDir .\artifacts\s23-http-smoke
```

Gate comparable:

```powershell
.\scripts\run_production_gate.ps1 -Frames 100000 -BatchSize 64 -Protocol tcp `
  -EvidenceDir .\artifacts\s23-tcp-100k
```

El gate comparable solo es `PASS` si reporta `frames_sent=100000`,
`frames_ok=100000`, `synthetic_cycle=false`, TLS, non-root y lifecycle/reconnect
correctos. Los smoke reducidos no sustituyen esta aceptación.

## Security

- Ningún secreto se pasa como argumento al launcher.
- Los secretos efímeros los sigue generando el script S22 dentro de su runtime.
- La evidencia no debe contener PEM, tokens, contenido de variables secretas ni
  credenciales de Docker.
- La guía no debe recomendar publicar puertos adicionales ni desactivar TLS o
  validaciones de seguridad.
- Los contenedores continúan ejecutándose como `10001:10001`.

## Verification and evidence

S23 añadirá contratos para los prerrequisitos, los cuatro comandos de ejecución,
los estados `NO EJECUTADA`/`FAIL`/`PASS`, el objetivo `100000` y la ausencia de
secretos en la evidencia. La ejecución real debe registrar:

- versión de Windows y WSL reportada por el operador;
- nombre y versión de la distribución Linux;
- versiones de Docker y Compose desde Windows y Linux;
- resultado de smoke TCP y HTTP;
- resultado del gate TCP de `100000` frames;
- hashes, duración y usuario efectivo sin material sensible;
- cualquier limitación o caso no ejecutado.

Si el entorno no puede prepararse, S23 termina con evidencia `NO EJECUTADA` y
el motivo exacto; no se declara soporte operativo por la existencia de una guía.

## Acceptance criteria

- Existe una guía reproducible para preparar WSL2 y Docker Desktop manualmente.
- El launcher S22 detecta una distro de usuario y puede invocar el gate desde
  WSL sin errores de encoding, rutas o quoting.
- Smoke TCP y HTTP se ejecutan y producen evidencia sanitizada.
- El gate comparable TCP acepta exactamente `100000` frames.
- Los contratos y suites existentes permanecen verdes.
- Los fallos de entorno quedan diferenciados de los fallos del producto.
- No se realizan instalaciones automáticas ni cambios globales del host.

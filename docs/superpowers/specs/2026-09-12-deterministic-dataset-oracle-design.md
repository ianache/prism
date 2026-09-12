# Diseño del generador determinista de dataset y oráculo de corrección

**Estado:** Diseño propuesto para revisión
**Fecha:** 2026-09-12
**Alcance:** Siguiente incremento posterior al freeze de especificación

## 1. Objetivo

Construir una herramienta de referencia, reproducible y auditable que genere los fixtures binarios del benchmark P0 y produzca sus resultados esperados mediante el contrato F1-F6. La herramienta será la autoridad de corrección para futuras implementaciones B0/B1 en Rust, Java y Go; no será parte del runtime de producción ni del camino medido.

## 2. Decisión arquitectónica

La implementación será Python 3 usando únicamente la biblioteca estándar. Se expondrá como módulos pequeños y una CLI:

- tools/dataset/prng.py: SplitMix64 con operaciones uint64 de wraparound.
- tools/dataset/frame.py: serialización, deserialización, rangos y CRC-32C del frame v1.
- tools/dataset/rules.py: evaluación de specification/rule-table.json con enteros.
- tools/dataset/oracle.py: pipeline F1-F6 y salida canónica.
- tools/dataset/generate.py: generación de fixtures, manifest y expected-results.
- tools/dataset/verify.py: verificación de digests, esquemas, conteos y regeneración.
- tests/dataset/: pruebas unitarias, de propiedades deterministas y de integración CLI.

No se usarán paquetes externos, reloj del sistema, locale, red, variables aleatorias del sistema ni aritmética de punto flotante. La elección de Python es deliberada: mantiene el oráculo legible y portable mientras los runtimes candidatos permanecen independientes.

## 3. Flujo de datos

La CLI genera primero una secuencia lógica de casos a partir del seed congelado, serializa cada caso a bytes según specification/telemetry-workflow.md, calcula SHA-256, ejecuta el oráculo y escribe:

1. datasets/fixtures/<fixture_id>.bin: bytes inmutables;
2. datasets/expected-results.jsonl: un resultado por fixture, en orden lexicográfico;
3. datasets/manifest.json: seed, versión, conteos, clases, algoritmo de digest y lista de fixtures;
4. datasets/manifest.sha256: digest del manifest canónico.

La generación se realiza en un directorio temporal y se publica mediante rename solo después de completar todas las verificaciones. Una ejecución parcial nunca debe parecer un dataset válido.

## 4. Contrato de generación

La semilla es 0x505249534D5F5631; el PRNG es SplitMix64 con wraparound uint64. El generador asigna N=1,000,000 casos: 800,000 valid, 50,000 invalid y 150,000 edge_complex. Para cualquier otro N usa largest remainder sobre valid=80%, edge_complex=15% e invalid=5%; los empates se resuelven en ese orden. Dentro de cada categoría, las clases 105, 249 y 501 bytes se asignan por round-robin comenzando en 105. El identificador es `p0-{validity_class}-{index:07d}-{payload_class:03d}` y bytes, JSONL y manifest se escriben en streaming en ese orden.

Los casos válidos se construyen con campos dentro de rango. Los inválidos se derivan de un caso válido aplicando una sola mutación nombrada: truncation, bad_magic, bad_version, length_mismatch, range_violation, unsupported_protocol o checksum_failure. Los edge/complex cubren igualdad de umbral, mínimos y máximos legales, sensor opcional ausente y área de sensores máxima.

## 5. Contrato de serialización

frame.py debe escribir exactamente la tabla de offsets congelada. La longitud total debe ser 105, 249 o 501 según la clase. El área de sensores usa registros de seis bytes y el CRC-32C se calcula sobre todos los bytes anteriores al checksum. La serialización solo acepta enteros dentro de rango y rechaza valores no enteros antes de producir bytes.

El orden de los sensores es estrictamente creciente por id. El resultado lógico conserva ese orden. La salida JSON usa UTF-8, LF, sin espacios innecesarios y claves ordenadas; los enteros se escriben como números JSON. El digest es SHA-256 de los bytes exactos del fixture.

## 6. Oráculo

El oráculo implementa la misma semántica, pero con límites explícitos y sin reutilizar código que una implementación B0/B1 deba compartir:

1. F1 verifica truncation, magic, version, length, protocol, ranges y checksum en ese orden.
2. F2 decodifica a una estructura de enteros y registros ordenados.
3. F3 conserva timestamp Unix en segundos, coordenadas e7, velocidad cm/s y heading cdeg.
4. F4 carga la tabla de reglas y aplica primera coincidencia con operadores inclusivos; una regla dependiente de sensor ausente no coincide.
5. F5 conserva label y severity de la coincidencia o usa NORMAL/INFO.
6. F6 conserva route STANDARD, ALERT o QUARANTINE sin efectos externos.

Un éxito produce kind=normalized_telemetry. Una entrada inválida produce kind=rejection con código, stage y contexto estable. Una excepción inesperada produce kind=execution_failure y se contabiliza separadamente.

## 7. CLI y comandos reproducibles

La interfaz será:

    python -m tools.dataset.generate --output datasets --seed 0x505249534D5F5631 --count 1000000
    python -m tools.dataset.verify --dataset datasets
    python -m tools.dataset.oracle --input datasets/fixtures --output datasets/expected-results.jsonl

generate debe rechazar un directorio de salida no vacío salvo que se indique una opción explícita de reemplazo para un directorio temporal. verify no modifica archivos. oracle solo escribe en una ruta temporal o en un archivo nuevo y falla si el resultado ya existe.

## 8. Pruebas

Las pruebas deben cubrir la secuencia conocida de SplitMix64, CRC-32C con vector conocido, offsets, longitudes 105/249/501, límites inclusivos, precedencia F1, igualdad de thresholds, sensor ausente, estabilidad byte-a-byte, sensibilidad a seed/version, conteos exactos, manifest, SHA-256, JSON Schema y separación entre rejection y execution_failure.

La prueba de integración debe generar un dataset pequeño de 300 fixtures en un directorio temporal y verificarlo sin escribir en el dataset canónico del repositorio.

## 9. Integridad y auditoría

El manifest registra dataset_id, generator_version, oracle_version, seed, PRNG, schema versions, payload classes, counts, fixture ordering, digest algorithm y cada fixture con fixture_id, category, class, filename, size, sha256 y expected_result_line. La generación no retiene la lista completa de fixtures en memoria. Cualquier diferencia de digest, conteo, orden o versión invalida la ejecución.

Los fixtures canónicos no se regeneran durante una corrida de benchmark. La generación y la verificación son pasos previos; el benchmark solo lee entradas inmutables.

## 10. No objetivos

Este incremento no implementa B0/B1 en Rust, Java o Go, no mide rendimiento, no incorpora brokers, no añade dependencias externas, no crea un runtime de producción y no modifica los contratos v1 congelados.

## 11. Criterios de aceptación

El incremento queda listo cuando una máquina limpia puede ejecutar generate y verify con los comandos documentados, producir resultados byte-a-byte repetibles, validar todos los esquemas, cubrir las categorías y mutaciones exigidas, y demostrar que el oráculo no depende de reloj, locale, red, RNG del sistema o punto flotante.

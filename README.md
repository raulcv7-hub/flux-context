# flux-context (CLI)

[![Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Context** es una herramienta de CLI de alto rendimiento diseñada para ingerir repositorios de código y documentación compleja, transformándolos en un contexto único, limpio y optimizado para ser consumido por LLMs (ChatGPT, Claude, Llama 3).

A diferencia de `cat` o scripts simples, Context entiende la estructura de tu proyecto, ignora el ruido, lee formatos binarios (PDF, Excel, Word) y te ofrece control total sobre qué enviar a la IA.

## Características Principales

* **Rendimiento Extremo:** Escrito en Rust, usa paralelismo de datos (`rayon`) para procesar miles de archivos en milisegundos.
* **Ingesta Pluri-Formato:** Soporte nativo para Code (`.rs`, `.py`, etc.), Documentos (`.pdf`, `.docx`) y Hojas de Cálculo (`.xlsx`).
* **Interfaz Interactiva (TUI):** Modo visual (`-I`) para seleccionar carpetas y archivos específicos navegando por un árbol.
* **Filtrado Inteligente:** Ignora automáticamente `node_modules`, `target`, `.git`, lockfiles y archivos binarios desconocidos.
* **Salida Versátil:** Genera reportes en **XML** (default), **Markdown**, **JSON** o **Texto Plano**.
* **Token Optimization:** Modo de minificación (`-m`) agresiva para ahorrar tokens en la ventana de contexto.
* **Clipboard Ready:** Copia el resultado directamente al portapapeles con `-c`.

## Instalación

### Desde el código fuente (Recomendado)

Necesitas tener [Rust instalado](https://rustup.rs/).

```bash
git clone https://github.com/TU_USUARIO/context-engine.git
cd context-engine
cargo install --path .
```

### Binarios Precompilados

Ve a la sección de [Releases](https://github.com/TU_USUARIO/context-engine/releases) y descarga el ejecutable para Windows, Linux o macOS.

## Uso Básico

El comando por defecto escanea el directorio actual y genera un XML en pantalla:

```bash
context
```

### Ejemplos Comunes

**1. Copiar contexto al portapapeles:**

```bash
context . -c
```

**2. Generar un Markdown para documentación:**

```bash
context . --format markdown -o reporte.md
```

**3. Modo Interactivo (Selección manual):**

```bash
context . -I
```

**4. Filtrar por extensión y excluir rutas:**

```bash
# Solo archivos Rust, ignorando la carpeta 'tests'
context . -e rs -X tests
```

**5. Incluir documentación PDF/Word y minificar:**

```bash
# Lee src y docs, minifica el código para ahorrar tokens
context . -i src -i docs -m -o contexto_full.xml
```

### Modo Interactivo (Panel de Control)

Al usar el flag `-I`, entras en una interfaz donde puedes navegar por el árbol de archivos.

```bash
context . -I
```

**Atajos en el modo interactivo:**

| Tecla | Acción |
| :--- | :--- |
| `Espacio` | Seleccionar / Deseleccionar archivo o carpeta. |
| `Enter` | **Confirmar y Ejecutar** con la configuración actual. |
| `c` | Alternar **Clipboard** (ON/OFF). |
| `m` | Alternar **Minificación** (ON/OFF). |
| `f` | Cambiar **Formato** (XML -> Markdown -> JSON -> Text). |
| `q` / `Esc` | Cancelar y Salir. |
| `Derecha` / `Izquierda` | Expandir o colapsar carpetas. |

## Opciones Avanzadas (CLI Flags)

| Flag | Descripción |
| :--- | :--- |
| `-o`, `--output <FILE>` | Guarda el resultado en un archivo específico. |
| `-c`, `--clip` | Copia el resultado al portapapeles automáticamente. |
| `--format <FMT>` | Formato: `xml` (default), `markdown`, `json`, `text`. |
| `-m`, `--minify` | Elimina indentación y líneas vacías (Ahorro de tokens). |
| `-I`, `--interactive` | Abre la interfaz TUI para selección y configuración manual. |
| `-e`, `--extensions` | Lista blanca de extensiones (ej: `rs,py`). |
| `-x`, `--exclude` | Lista negra de extensiones (ej: `lock,png`). |
| `-i`, `--include-path` | Solo incluye rutas que contengan este texto. |
| `-X`, `--exclude-path` | Excluye rutas que contengan este texto. |
| `-d`, `--depth <N>` | Máxima profundidad de escaneo en directorios. |
| `--include-hidden` | Incluye archivos ocultos (empezados por punto). |
| `--no-ignore` | Ignora los archivos `.gitignore` y `.ignore`. |
| `-v`, `--verbose` | Muestra logs de depuración (usar `-vv` para más detalle). |

## Arquitectura

El proyecto sigue una **Arquitectura Hexagonal** para garantizar testabilidad y mantenibilidad.

```text
src/
├── core/       # Lógica de Negocio Pura (Config, Entidades, Minificación)
├── ports/      # Interfaces (Traits) para Scanner, Reader, Writer
└── adapters/   # Implementaciones Reales
    ├── fs_scanner.rs  # Motor 'ignore' (ripgrep)
    ├── parsers/       # Estrategias (PDF, DOCX, Excel)
    ├── output/        # Formateadores (XML, JSON, MD)
    └── ui/            # TUI con Ratatui
```

## Contribución

1. Haz un Fork del repositorio.
2. Crea una rama (`git checkout -b feat/nueva-funcionalidad`).
3. Haz tus cambios y asegúrate de pasar los tests (`cargo test`).
4. Haz Push a la rama y abre un Pull Request.

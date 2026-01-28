# Context Engine

> **Misión:** "Proveer a la Inteligencia Artificial de la misma visibilidad que tiene el humano, instantáneamente."

**Context** es una herramienta de CLI de alto rendimiento escrita en **Rust**. Su único propósito es ingerir carpetas de proyectos complejos (código, documentación, datos, binarios de oficina) y serializarlos en un **Contexto Unificado y Limpio** optimizado para ser consumido por LLMs (GPT-4, Claude 3, Llama 3).

A diferencia de herramientas simples como `cat` o `tree`, Context entiende la semántica de un repositorio: sabe qué es ruido, sabe leer PDFs y Excel, y sabe cómo presentar la información para maximizar la ventana de contexto del LLM.

---

## 1. CAPACIDADES PRINCIPALES (Core Capabilities)

### 1.1. Ingesta Omni-Formato
Context no solo lee código. Un proyecto real tiene especificaciones en Word y datos en Excel. Context los unifica.
*   **Código Fuente:** Soporte nativo UTF-8 para cualquier lenguaje de programación.
*   **Documentación Estructurada:** Markdown (`.md`), ReStructuredText (`.rst`), Texto plano.
*   **Capa "Office" (Binary extraction):**
    *   **PDF:** Extracción de texto plano y estructura básica.
    *   **DOCX:** Extracción de cuerpo de texto ignorando estilos de formato ruido.
    *   **Excel/CSV:** Volcado de hojas de cálculo a representaciones CSV/Markdown tabulares.
    *   **Imágenes (Futuro):** OCR ligero para diagramas de arquitectura.

### 1.2. El Gran Filtro (The Great Filter)
El valor de Context no es solo lo que incluye, sino lo que **excluye**. Para un LLM, el ruido es coste y alucinación.
Context implementa un sistema de listas de bloqueo agresivo y adaptativo para:
*   **Web Moderno:** `node_modules`, `.next`, `build`, `dist`, `coverage`.
*   **Python/AI:** `__pycache__`, `.venv`, `venv`, `.mypy_cache`, checkpoints (`.pt`, `.safetensors`).
*   **Sistemas/IDE:** `.DS_Store`, `.idea`, `.vscode`.
*   **Logs y Temporales:** `*.log`, `*.tmp`, `*.bak`.
*   **Bloqueo Binario Heurístico:** Detección automática de archivos binarios desconocidos para evitar corromper el prompt.

### 1.3. Reporte de Inteligencia (Meta-Context)
Al final del volcado, Context genera un **Resumen Ejecutivo** para el LLM:
*   **Topología:** Un árbol ASCII (`tree`) del directorio filtrado para que el LLM entienda la ubicación física.
*   **Estadísticas:** Conteo total de archivos, tamaño total en tokens (estimado), y desglose porcentual por lenguaje (ej: "40% Rust, 30% Python, 20% Docs").

---

## 2. FORMATO DE SALIDA (The Output Protocol)

El output es un solo archivo (XML-wrapped) diseñado para que el LLM pueda parsear límites de archivos inequívocamente.

```xml
<Context_context>
    <metadata>
        <project_root>/home/user/projects/my-app</project_root>
        <scan_time>2026-01-28T10:00:00Z</scan_time>
        <stats>
            <total_files>42</total_files>
            <total_tokens_approx>15000</total_tokens_approx>
            <languages>
                <lang name="Rust" percent="60" />
                <lang name="Markdown" percent="40" />
            </languages>
        </stats>
    </metadata>

    <directory_structure>
        src/
        ├── main.rs
        └── utils.rs
        docs/
        └── specs.pdf
    </directory_structure>

    <files>
        <file path="src/main.rs" type="code" lang="rust">
            <![CDATA[
            fn main() {
                println!("Hello World");
            }
            ]]>
        </file>

        <file path="docs/specs.pdf" type="document" lang="text">
            <![CDATA[
            [PDF CONTENT EXTRACTED]
            Specification 1.0...
            ]]>
        </file>
    </files>
</Context_context>
```

---

## 3. ESTRATEGIA DE EXCLUSIÓN (The Ignore Matrix)

Context aplica reglas de exclusión en capas de prioridad:

### Nivel 1: Hard System Ignores (Innegociables)
*   **VCS:** `.git`, `.svn`, `.hg`.
*   **System:** `.DS_Store`, `Thumbs.db`.
*   **Lockfiles Binarios:** `package-lock.json` (Opcional, a veces es ruido masivo), `yarn.lock` (si es gigante).

### Nivel 2: Ecosistemas Detectados
Context detecta el tipo de proyecto y aplica filtros específicos:

| Ecosistema | Patrones Ignorados |
| :--- | :--- |
| **Node/Web** | `node_modules`, `bower_components`, `jspm_packages`, `.npm`, `build`, `dist`, `.cache`, `.parcel-cache` |
| **Python** | `__pycache__`, `*.py[cod]`, `.venv`, `venv`, `env`, `.tox`, `.nox`, `*.egg-info`, `instance` |
| **Java/JVM** | `target`, `build`, `.gradle`, `*.class`, `*.jar`, `*.war` |
| **Rust** | `target` (debug/release), `Cargo.lock` (opcional) |
| **C/C++** | `cmake-build-debug`, `CMakeFiles`, `*.o`, `*.obj`, `*.so`, `*.dll`, `*.exe` |
| **AI/ML** | `wandb`, `runs`, `results`, `*.ckpt`, `*.pt`, `*.pth`, `*.onnx`, `*.safetensors` (Modelos pesados) |

### Nivel 3: User Overrides
*   Respeto estricto a `.gitignore` del proyecto (usando la crate `ignore` de Rust).
*   Soporte para `.Contextignore` específico.

---

## 4. OPTIMIZACIONES DE TOKEN (Roadmap)

En versiones avanzadas, Context no solo volcará texto, sino que lo **comprimirá semánticamente**:

1.  **Code Minification (Safe):** Eliminación de espacios en blanco redundantes y comentarios vacíos en tiempo de volcado.
2.  **Stopwords Removal (Docs):** En archivos PDF/Word, eliminar palabras vacías ("el", "la", "un") si se activa el modo "High Compression".
3.  **Skeleton Mode:** Para archivos gigantes, volcar solo las firmas de funciones y clases (outline), omitiendo el cuerpo de la implementación.

---

## 5. ARQUITECTURA TÉCNICA (Rust Stack)

El proyecto se construirá sobre el ecosistema de alto rendimiento de Rust:

*   **CLI:** `clap` (Argument parsing robusto).
*   **Concurrency:** `rayon` (Procesamiento paralelo de archivos. Leer 1000 archivos en milisegundos).
*   **Traversal:** `ignore` (Librería de `ripgrep` para caminar directorios respetando gitignore).
*   **Parsing:**
    *   `pdf-extract` / `lopdf` (PDF).
    *   `docx-rs` / `zip` (Word).
    *   `calamine` (Excel eficiente).
*   **Output:** `quick-xml` o escritura directa a buffer (`std::io::BufWriter`) para velocidad máxima.

---

## 6. Requisitos

### 1. Interfaz de Línea de Comandos (CLI) & Configuración

*   **FR-CLI-001 (Target Selection):** El sistema debe aceptar una ruta de directorio o un archivo individual como argumento posicional. Si no se provee, debe asumir el directorio actual (`.`).
*   **FR-CLI-002 (Output Destination):** Debe permitir especificar un archivo de salida (flag `-o` / `--output`).
    *   Si no se especifica, debe soportar imprimir a `stdout` (para piping: `codex . | clipboard`).
*   **FR-CLI-003 (Verbosity Levels):** Debe implementar niveles de log (`-v` para info, `-vv` para debug de parseo, `-q` para silencio total).
*   **FR-CLI-004 (Configuration File):** Debe buscar automáticamente un archivo de configuración (`codex.toml` o `.codexignore`) en la raíz del proyecto para cargar reglas persistentes.
*   **FR-CLI-005 (Token Limit Warning):** Debe permitir definir un límite de tokens (`--max-tokens`). Si el contexto generado lo supera, debe emitir una advertencia (pero generar el archivo igualmente).
*   **FR-CLI-006 (Clipboard Integration):** Debe incluir un flag (`--clip` o `-c`) para copiar el resultado directamente al portapapeles del sistema (usando crates como `arboard`).

### 2. Motor de Descubrimiento y Recorrido (Traversal Engine)

*   **FR-TRV-001 (Recursive Walk):** El sistema debe recorrer recursivamente el árbol de directorios desde la ruta raíz.
*   **FR-TRV-002 (Gitignore Compliance):** Debe leer y respetar obligatoriamente los archivos `.gitignore` (global y local).
*   **FR-TRV-003 (Hidden Files Policy):** Por defecto, debe ignorar archivos y carpetas ocultos (`.git`, `.env`, `.config`), salvo que se fuerce su inclusión con `--include-hidden`.
*   **FR-TRV-004 (Vendor Detection):** Debe detectar y excluir automáticamente directorios de dependencias conocidos sin necesidad de configuración (`node_modules`, `venv`, `.venv`, `target`, `dist`, `build`, `vendor`).
*   **FR-TRV-005 (Max Depth):** Debe permitir limitar la profundidad de recursión (flag `--depth`).
*   **FR-TRV-006 (Symlink Handling):** Debe tener una política configurable para enlaces simbólicos (por defecto: ignorar para evitar ciclos infinitos, flag `--follow-symlinks` para activar).

### 3. Sistema de Filtrado y Selección (The Great Filter)

*   **FR-FLT-001 (Binary Detection):** Debe analizar los primeros bytes de cada archivo ("Magic Bytes") para determinar si es binario.
    *   Los archivos binarios detectados deben ser excluidos del contenido textual.
    *   Se debe generar una entrada en el reporte indicando que el archivo existe pero es binario.
*   **FR-FLT-002 (Extension Whitelist/Blacklist):** Debe permitir filtrar por extensión (ej: `codex . --only rs,py` o `codex . --exclude json`).
*   **FR-FLT-003 (Size Limit Hard Cap):** Debe ignorar archivos que superen un tamaño máximo (ej: 10MB) para proteger la memoria y el contexto del LLM.
*   **FR-FLT-004 (Lockfile Policy):** Debe tener lógica especial para `package-lock.json`, `yarn.lock`, `Cargo.lock`. Por defecto deben excluirse o truncarse si son demasiado grandes, ya que aportan poco valor semántico y mucho ruido.

### 4. Motor de Ingesta y Parsing (The Parsing Core)

*   **FR-PRS-001 (Text Fallback):** Debe intentar decodificar archivos de texto usando `UTF-8`. Si falla, debe intentar una lista de fallbacks (`Latin-1`, `Windows-1252`) antes de desistir.
*   **FR-PRS-002 (PDF Parsing):** Debe ser capaz de extraer texto plano de archivos `.pdf`, preservando un mínimo de estructura (saltos de página/línea).
*   **FR-PRS-003 (DOCX Parsing):** Debe extraer texto de archivos Microsoft Word (`.docx`), ignorando estilos de fuente pero preservando el contenido legible.
*   **FR-PRS-004 (Spreadsheet Parsing):** Debe leer archivos `.csv` y `.xlsx`.
    *   El contenido debe renderizarse como una tabla Markdown o CSV textual para que el LLM pueda interpretarlo.
    *   Debe tener un límite de filas (ej: primeras 50 filas) para no inundar el contexto con datos masivos.
*   **FR-PRS-005 (Image Placeholder):** Para archivos de imagen (`.png`, `.jpg`), debe generar un marcador en el output `[IMAGE: logo.png - 150KB]` en lugar del contenido binario (Prepración para futuro OCR).

### 5. Procesamiento y Optimización de Contenido

*   **FR-OPT-001 (Token Counting):** Debe calcular el número de tokens (usando el encoding `cl100k_base` de OpenAI o similar) para cada archivo y para el total del proyecto.
*   **FR-OPT-002 (Content Truncation):** Si un archivo de texto excede un límite configurable (ej: 100k tokens), debe truncar el centro y dejar el inicio y el final (`[... 5000 lines truncated ...]`), ya que el principio (imports) y el final suelen ser lo más relevante.
*   **FR-OPT-003 (Whitespace Reduction - Optional):** Debe incluir un modo `--minify` que elimine líneas vacías consecutivas excesivas para ahorrar tokens.

### 6. Generación de Salida (The Output Formatter)

*   **FR-OUT-001 (XML Structure):** El formato de salida principal debe ser XML-like para facilitar el parsing por parte de los LLMs.
    *   Estructura: `<file path="src/main.rs"> ... contenido ... </file>`.
*   **FR-OUT-002 (CDATA Escaping):** Todo contenido de archivo debe estar envuelto en bloques `<![CDATA[ ... ]]>` para evitar romper la estructura XML si el código contiene caracteres `<` o `>`.
*   **FR-OUT-003 (Project Tree Map):** El inicio del archivo de salida debe contener obligatoriamente una representación en árbol ASCII (`tree`) de todos los archivos incluidos en el contexto. Esto da al LLM "consciencia espacial".
*   **FR-OUT-004 (Metadata Header):** Debe incluir una cabecera con:
    *   Fecha de escaneo.
    *   Ruta raíz absoluta.
    *   Estadísticas de tokens y número de archivos.
    *   Lista de extensiones encontradas (Top lenguajes).
*   **FR-OUT-005 (Markdown Alternative):** Debe permitir un flag `--format markdown` que genere bloques de código con triple backtick (```` ```rust ... ``` ````) para ser copiado directamente a un chat humano si se prefiere.

### 7. Rendimiento y Concurrencia

*   **FR-PERF-001 (Parallel Processing):** La lectura de archivos y el conteo de tokens deben realizarse en paralelo (Multi-threading) para aprovechar todos los núcleos del CPU.
*   **FR-PERF-002 (Streaming Output):** Para salidas muy grandes, el sistema debe ser capaz de escribir al buffer de salida (`stdout` o archivo) incrementalmente, minimizando el uso de RAM (no cargar todo el string gigante en memoria antes de escribir).

### 8. Gestión de Errores y Robustez

*   **FR-ERR-001 (Permission Fault Tolerance):** Si el sistema encuentra un archivo/directorio sin permisos de lectura (`EACCES`), debe registrar un warning, saltarlo y continuar, nunca detener la ejecución.
*   **FR-ERR-002 (Corrupt File Handling):** Si un parseador (PDF/Excel) falla o entra en pánico, el error debe ser capturado, el archivo marcado como `[ERROR READING FILE]`, y el proceso debe continuar con el resto.
*   **FR-ERR-003 (Symlink Cycles):** Debe detectar ciclos en enlaces simbólicos y abortar esa rama de recursión para evitar bucles infinitos.

---

## 7 Future

- Visualización del Árbol: Tu generate_tree actual es una lista plana (├── src/main.rs). Un árbol real anidado (como el comando tree de Linux) ahorra tokens y el LLM entiende mejor la profundidad.
- Seguridad de Caracteres: quick-xml a veces falla con caracteres de control extraños en binarios que se cuelan como texto. Se debería sanitizar el string antes de meterlo en CDATA.

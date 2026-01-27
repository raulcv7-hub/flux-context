# GUÍA SOBRE DECISIÓN ARQUITECTÓNICA

**Proyecto:** `Context Engine (CLI)`
**Estado:** `[Vigente]`
**Última Actualización:** `28/01/2026`

**Propósito:** Este documento actúa como el "Framework de Referencia" para el equipo de ingeniería. Define no solo *qué* tecnologías usamos, sino *por qué* y *cómo* se estructuran.
**Instrucciones:** Las decisiones se toman en cascada. Una elección en el Nivel 1 restringe las opciones disponibles en los niveles inferiores.

---

## 0. NIVEL PRE-REQUISITOS: Restricciones del Mundo Real
*Antes de hablar de código, definimos el contexto que nos limita.*

*   **Equipo:** 1 Ingeniero
*   **Budget/Infra:** Ejecución local (CLI). -> *Recursos: CPU Multicore y RAM del usuario.*
*   **Compliance:** Open Source (MIT/Apache). -> *Código limpio y publicable en GitHub.*
*   **Time-to-Market:** Calidad > Velocidad. -> *Se priorizan los tests y la refactorización sobre features rápidas.*

---

## 1. NIVEL SISTEMA: Topología Física (The Shape)
*¿Cómo se despliega y escala el sistema a vista de pájaro?*

| Topología | Descripción | Contexto Ideal | Trade-offs (El precio a pagar) |
| :--- | :--- | :--- | :--- |
| **Monolito Modular** | Un solo ejecutable/proceso con módulos internos estrictos. | **ELEGIDO.** Herramientas CLI robustas. | Escalado limitado a la máquina local (Vertical). |

*   **Justificación:** Rust compila a un binario estático único. La modularidad será interna (Crates/Módulos) para separar responsabilidades (Parsing vs CLI vs Core).

---

## 2. NIVEL DATOS: Estrategia de Estado (Data Scope)
*El código es efímero, los datos pesan. ¿Cómo gestionamos la verdad?*

### 2.1. Propiedad del Dato
*   Conceptualmente, la "Base de Datos" es el **Sistema de Archivos (File System)** del usuario (Solo Lectura) y la configuración en memoria.

### 2.2. Consistencia
*   **Snapshot Inmutable:** Al iniciar la ejecución, el estado del directorio se considera la "verdad". No reaccionamos a cambios en tiempo real (watch mode) en esta versión (MVP).

### 2.3. Modelo de Persistencia
*   **Stateless (Efímero):** La aplicación carga, procesa y muere.
*   **In-Memory Graph:** Construiremos una representación en memoria del árbol de archivos antes de serializarla.

---

## 3. NIVEL ESTRUCTURAL: Organización Lógica (Logical Scope)
*¿Cómo aislamos el negocio de la tecnología dentro del código?*

| Patrón | Enfoque | Cuándo elegirlo | Coste |
| :--- | :--- | :--- | :--- |
| **Hexagonal (Ports & Adapters)** | Dominio <- Interfaces -> Infra | **ELEGIDO.** Permite testear lógica sin tocar disco real. | Boilerplate de Traits y DTOs. |

*   **Adaptación al Proyecto:**
    *   **Dominio (Core):** Reglas de filtrado, lógica de conteo de tokens, estructura del árbol. (Puro Rust, sin I/O).
    *   **Puertos (Interfaces):** `FileSystemScanner`, `TokenCounter`, `ReportWriter`.
    *   **Adaptadores (Infra):** Implementación real con `std::fs`, implementación de `OpenAI Tokenizer`, implementación de `Stdout`.

---

## 4. NIVEL IMPLEMENTACIÓN: Patrones Tácticos (Code Scope)
*Herramientas estandarizadas para resolver problemas recurrentes.*

*   **Comunicación entre Módulos:**
    *   `[X]` Síncrona / Paralelismo de Datos (Data Parallelism). Usaremos `Rayon` para procesar listas de archivos en paralelo.
*   **Manejo de Errores:**
    *   `[X]` Result Pattern (`Result<T, E>`).
        *   **App:** `anyhow::Result` para manejo flexible en el `main`.
        *   **Libs:** `thiserror` para errores tipados y específicos en el dominio.
*   **Principios Rectores:**
    *   **SRP (Single Responsibility):** Un parser de PDF no debe saber contar tokens.
    *   **Type State Pattern:** Usar el sistema de tipos de Rust para impedir estados inválidos (ej: `UnscannedProject` vs `ScannedProject`).

---

## 5. NIVEL OPERATIVO: Arquitectura de Despliegue (Ops Scope)
*Un sistema no existe hasta que corre en producción.*

### 5.1. Estrategia de Despliegue
*   `[X]` **Binary Distribution:** Compilación cruzada o local (`cargo build --release`).

### 5.2. Observabilidad (Los 3 Pilares)
*   **Logs:** `tracing` crate. Niveles: `INFO` (progreso usuario), `DEBUG` (detalle parser), `TRACE` (byte-level).
*   **Métricas:** Salida final de estadísticas (Tiempo de ejecución, tokens/segundo).

### 5.3. Resiliencia
*   **Fault Tolerance:** Si un archivo falla al leerse (permisos, corrupción), se registra el error y **se continúa**. Nunca pánico (panic) en el hilo principal.

---

## 6. ATRIBUTOS DE CALIDAD: Los Trade-offs (Cross-Cutting)
*No se puede tener todo. Elegimos nuestros superpoderes y nuestras debilidades.*

> **REGLA:** Elige un máximo de **2 prioridades críticas**. El resto serán secundarias.

1.  `[X]` **Mantenibilidad:** Código idiomático Rust, separación estricta para facilitar el aprendizaje y la contribución.
2.  `[X]` **Performance:** El objetivo es procesar repositorios grandes en milisegundos.

*Sacrificamos:* **Simplicidad inicial** (Hexagonal es más complejo de arrancar que un script lineal) en favor de testabilidad a largo plazo.

---

## 7. GOBERNANZA Y EVOLUCIÓN
*¿Cómo cambia este documento?*

Para modificar una decisión arquitectónica establecida se debe seguir el proceso de **RFC**:

1.  Crear un documento proponiendo el cambio.
2.  Defender el "Por qué" (Problema actual) y el "Cómo" (Coste de migración).
3.  Si se aprueba, se genera un **ADR (Architecture Decision Record)**.

---

### Anexo: Glosario de Tecnologías Elegidas (Stack)
*Rellena aquí las herramientas finales que materializan la arquitectura.*

*   **Lenguaje:** Rust (Edition 2024)
*   **CLI Framework:** `clap` (derive feature)
*   **Paralelismo:** `rayon`
*   **Logging:** `tracing` + `tracing-subscriber`
*   **Filesystem Walk:** `ignore` (motor de ripgrep)
*   **Testing:** `cargo test` (unit), `assert_cmd` (integration CLI).
*   **Error Handling:** `anyhow` (bin), `thiserror` (lib).

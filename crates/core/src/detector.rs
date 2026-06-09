//! Detección inteligente de tipos de output para renderizado mejorado
//!
//! Este módulo detecta automáticamente diferentes tipos de contenido en el output:
//! - Logs (INFO, WARN, ERROR, DEBUG)
//! - Errores de compilación/runtime
//! - JSON
//! - Tablas ASCII
//! - Stack traces
//!
//! El sistema está diseñado para no interferir con scripts o pipelines.

/// Tipo de contenido detectado en una línea
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    /// Línea normal sin formato especial
    Normal,
    /// Mensaje de log (INFO, WARN, ERROR, DEBUG, TRACE)
    Log(LogLevel),
    /// Mensaje de error (compilación, runtime, etc.)
    Error,
    /// Línea que contiene JSON
    Json,
    /// Parte de una tabla ASCII
    Table,
    /// Stack trace (Java, Rust, Python, Node.js, etc.)
    StackTrace,
    /// Advertencia (warning, caution, etc.)
    Warning,
    /// Éxito (success, ok, passed, etc.)
    Success,
}

/// Nivel de log detectado
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

/// Información sobre un fragmento de JSON detectado
#[derive(Debug, Clone)]
pub struct JsonFragment {
    /// Posición inicial en la línea
    pub start_col: usize,
    /// Posición final en la línea
    pub end_col: usize,
    /// Nivel de profundidad de anidación
    pub depth: usize,
    /// Tipo de token
    pub token_type: JsonTokenType,
}

/// Tipos de tokens JSON
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonTokenType {
    BraceOpen,
    BraceClose,
    BracketOpen,
    BracketClose,
    String,
    Number,
    Boolean,
    Null,
    Key,
    Colon,
    Comma,
}

/// Información sobre una tabla detectada
#[derive(Debug, Clone)]
pub struct TableInfo {
    /// Columnas detectadas (posiciones de separadores)
    pub columns: Vec<usize>,
    /// Tipo de línea de la tabla
    pub row_type: TableRowType,
}

/// Tipo de fila en una tabla
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableRowType {
    /// Fila de encabezado
    Header,
    /// Fila de datos
    Data,
    /// Línea separadora
    Separator,
}

/// Detector de contenido con estado
pub struct ContentDetector {
    /// Estado de tablas multi-línea
    table_state: Option<TableState>,
}

#[derive(Debug)]
struct TableState {
    column_positions: Vec<usize>,
    rows_seen: usize,
}

impl Default for ContentDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentDetector {
    pub fn new() -> Self {
        Self { table_state: None }
    }

    /// Detecta el tipo de contenido en una línea
    pub fn detect_line(&mut self, line: &str) -> ContentType {
        let trimmed = line.trim_start();

        // Prioridad 1: Stack traces
        if is_stack_trace_line(trimmed) {
            return ContentType::StackTrace;
        }

        // Prioridad 2: Logs estructurados
        if let Some(level) = detect_log_level(trimmed) {
            return ContentType::Log(level);
        }

        // Prioridad 3: Errores
        if is_error_line(trimmed) {
            return ContentType::Error;
        }

        // Prioridad 4: Warnings
        if is_warning_line(trimmed) {
            return ContentType::Warning;
        }

        // Prioridad 5: Success
        if is_success_line(trimmed) {
            return ContentType::Success;
        }

        // Prioridad 6: JSON
        if is_json_line(line) {
            return ContentType::Json;
        }

        // Prioridad 7: Tablas
        if let Some(_table_info) = self.detect_table(line) {
            return ContentType::Table;
        }

        ContentType::Normal
    }

    /// Detecta si una línea es parte de una tabla
    fn detect_table(&mut self, line: &str) -> Option<TableInfo> {
        // Buscar separadores comunes en tablas
        if is_table_separator(line) {
            let columns = find_column_positions(line);
            self.table_state = Some(TableState {
                column_positions: columns.clone(),
                rows_seen: 0,
            });
            return Some(TableInfo {
                columns,
                row_type: TableRowType::Separator,
            });
        }

        // Si ya estamos en una tabla, verificar si esta línea pertenece
        if let Some(ref mut state) = self.table_state {
            if is_table_row(line, &state.column_positions) {
                state.rows_seen += 1;
                let row_type = if state.rows_seen == 1 {
                    TableRowType::Header
                } else {
                    TableRowType::Data
                };
                return Some(TableInfo {
                    columns: state.column_positions.clone(),
                    row_type,
                });
            } else {
                // No coincide, salir del estado de tabla
                self.table_state = None;
            }
        }

        None
    }

    /// Analiza una línea JSON y retorna fragmentos coloreables
    pub fn parse_json_fragments(&self, line: &str) -> Vec<JsonFragment> {
        let mut fragments = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        let mut depth = 0;
        let mut in_string = false;
        let mut is_key = false;

        while i < chars.len() {
            let c = chars[i];

            if in_string {
                if c == '"' && (i == 0 || chars[i - 1] != '\\') {
                    in_string = false;
                    // Determinar si era una key o un string value
                    let token_type = if is_key {
                        JsonTokenType::Key
                    } else {
                        JsonTokenType::String
                    };
                    fragments.push(JsonFragment {
                        start_col: i,
                        end_col: i + 1,
                        depth,
                        token_type,
                    });
                    is_key = false;
                }
            } else {
                match c {
                    '{' => {
                        fragments.push(JsonFragment {
                            start_col: i,
                            end_col: i + 1,
                            depth,
                            token_type: JsonTokenType::BraceOpen,
                        });
                        depth += 1;
                    }
                    '}' => {
                        depth = depth.saturating_sub(1);
                        fragments.push(JsonFragment {
                            start_col: i,
                            end_col: i + 1,
                            depth,
                            token_type: JsonTokenType::BraceClose,
                        });
                    }
                    '[' => {
                        fragments.push(JsonFragment {
                            start_col: i,
                            end_col: i + 1,
                            depth,
                            token_type: JsonTokenType::BracketOpen,
                        });
                        depth += 1;
                    }
                    ']' => {
                        depth = depth.saturating_sub(1);
                        fragments.push(JsonFragment {
                            start_col: i,
                            end_col: i + 1,
                            depth,
                            token_type: JsonTokenType::BracketClose,
                        });
                    }
                    '"' => {
                        in_string = true;
                        // Mirar hacia adelante para ver si es una key
                        is_key = peek_for_colon(&chars, i + 1);
                    }
                    ':' => {
                        fragments.push(JsonFragment {
                            start_col: i,
                            end_col: i + 1,
                            depth,
                            token_type: JsonTokenType::Colon,
                        });
                    }
                    ',' => {
                        fragments.push(JsonFragment {
                            start_col: i,
                            end_col: i + 1,
                            depth,
                            token_type: JsonTokenType::Comma,
                        });
                    }
                    't' | 'f'
                        // true o false
                        if (i + 4 <= chars.len()
                            && &chars[i..i + 4].iter().collect::<String>() == "true"
                            || i + 5 <= chars.len()
                                && &chars[i..i + 5].iter().collect::<String>() == "false")
                        => {
                            let len = if chars[i + 1] == 'r' { 4 } else { 5 };
                            fragments.push(JsonFragment {
                                start_col: i,
                                end_col: i + len,
                                depth,
                                token_type: JsonTokenType::Boolean,
                            });
                            i += len - 1;
                        }
                    'n'
                        // null
                        if i + 4 <= chars.len()
                            && &chars[i..i + 4].iter().collect::<String>() == "null"
                        => {
                            fragments.push(JsonFragment {
                                start_col: i,
                                end_col: i + 4,
                                depth,
                                token_type: JsonTokenType::Null,
                            });
                            i += 3;
                        }
                    '0'..='9' | '-' => {
                        // Número
                        let start = i;
                        while i < chars.len()
                            && (chars[i].is_numeric()
                                || chars[i] == '.'
                                || chars[i] == '-'
                                || chars[i] == 'e'
                                || chars[i] == 'E')
                        {
                            i += 1;
                        }
                        fragments.push(JsonFragment {
                            start_col: start,
                            end_col: i,
                            depth,
                            token_type: JsonTokenType::Number,
                        });
                        i -= 1;
                    }
                    _ => {}
                }
            }
            i += 1;
        }

        fragments
    }
}

// === Funciones auxiliares de detección ===

/// Detecta el nivel de log en una línea
fn detect_log_level(line: &str) -> Option<LogLevel> {
    let upper = line.to_uppercase();

    // Patrones comunes de logs
    if upper.contains("FATAL") || upper.contains("CRITICAL") {
        return Some(LogLevel::Fatal);
    }
    if upper.contains("ERROR") || upper.contains("[ERR]") || upper.contains("✗") {
        return Some(LogLevel::Error);
    }
    if upper.contains("WARN") || upper.contains("WARNING") || upper.contains("[WRN]") {
        return Some(LogLevel::Warn);
    }
    if upper.contains("INFO") || upper.contains("[INF]") || upper.contains("ℹ") {
        return Some(LogLevel::Info);
    }
    if upper.contains("DEBUG") || upper.contains("[DBG]") {
        return Some(LogLevel::Debug);
    }
    if upper.contains("TRACE") || upper.contains("[TRC]") {
        return Some(LogLevel::Trace);
    }

    None
}

/// Detecta si una línea es un stack trace
fn is_stack_trace_line(line: &str) -> bool {
    // Rust: "at src/main.rs:42:5" o "   --> src/main.rs:42:5"
    if line.contains("at ")
        && (line.contains(".rs:")
            || line.contains(".java:")
            || line.contains(".py:")
            || line.contains(".js:"))
    {
        return true;
    }
    if line.contains("-->") && line.contains(":") {
        return true;
    }

    // Java: "at com.example.Main.method(Main.java:42)"
    if line.trim_start().starts_with("at ") && line.contains("(") && line.contains(".java:") {
        return true;
    }

    // Python: 'File "main.py", line 42, in function'
    if line.contains("File \"") && line.contains("\", line ") {
        return true;
    }

    // JavaScript/Node.js: "at Function.name (file.js:42:10)"
    if line.trim_start().starts_with("at ") && (line.contains(".js:") || line.contains(".ts:")) {
        return true;
    }

    false
}

/// Detecta si una línea es un error
fn is_error_line(line: &str) -> bool {
    let lower = line.to_lowercase();

    // Patrones comunes de error
    lower.contains("error:")
        || lower.contains("exception")
        || lower.contains("traceback")
        || lower.contains("fatal:")
        || lower.starts_with("error[")
        || lower.contains("✗ error")
        || lower.contains("❌")
}

/// Detecta si una línea es una advertencia
fn is_warning_line(line: &str) -> bool {
    let lower = line.to_lowercase();

    lower.contains("warning:")
        || lower.contains("caution")
        || lower.starts_with("warning[")
        || lower.contains("⚠")
}

/// Detecta si una línea indica éxito
fn is_success_line(line: &str) -> bool {
    let lower = line.to_lowercase();

    lower.contains("✓")
        || lower.contains("✔")
        || lower.contains("success")
        || lower.contains("passed")
        || lower.contains("✅")
        || (lower.contains("ok") && !lower.contains("error"))
}

/// Detecta si una línea contiene JSON
fn is_json_line(line: &str) -> bool {
    let trimmed = line.trim();

    // JSON object o array
    if (trimmed.starts_with('{') || trimmed.starts_with('['))
        && (trimmed.ends_with('}') || trimmed.ends_with(']') || trimmed.ends_with(','))
    {
        return true;
    }

    // Línea con estructura JSON dentro
    if trimmed.contains(":{") || trimmed.contains(":[") {
        return true;
    }

    // Propiedades JSON comunes
    if trimmed.contains("\":") && (trimmed.contains('{') || trimmed.contains('}')) {
        return true;
    }

    false
}

/// Detecta si una línea es un separador de tabla
fn is_table_separator(line: &str) -> bool {
    let trimmed = line.trim();

    // Separadores comunes: +---+---+, |---|---|, ===, ---
    let separator_chars = ['+', '-', '=', '|', '─', '━', '═'];

    if trimmed.is_empty() {
        return false;
    }

    let non_whitespace: Vec<char> = trimmed.chars().filter(|c| !c.is_whitespace()).collect();

    // Debe tener al menos 3 caracteres de separador
    if non_whitespace.len() < 3 {
        return false;
    }

    // Al menos 70% de los caracteres no-whitespace deben ser separadores
    let separator_count = non_whitespace
        .iter()
        .filter(|c| separator_chars.contains(c))
        .count();

    (separator_count as f32 / non_whitespace.len() as f32) >= 0.7
}

/// Encuentra las posiciones de las columnas en un separador de tabla
fn find_column_positions(line: &str) -> Vec<usize> {
    let mut positions = Vec::new();

    for (i, c) in line.chars().enumerate() {
        if c == '+' || c == '|' {
            positions.push(i);
        }
    }

    positions
}

/// Verifica si una línea es una fila de tabla basada en las columnas conocidas
fn is_table_row(line: &str, column_positions: &[usize]) -> bool {
    if column_positions.is_empty() {
        return false;
    }

    // Verificar si tiene separadores cerca de las posiciones esperadas
    let chars: Vec<char> = line.chars().collect();
    let mut matches = 0;

    for &pos in column_positions {
        // Permitir cierta variación (±2 caracteres)
        for offset in -2..=2 {
            let check_pos = (pos as i32 + offset) as usize;
            if check_pos < chars.len() && chars[check_pos] == '|' {
                matches += 1;
                break;
            }
        }
    }

    // Si coincide al menos el 50% de las columnas, es una fila de tabla
    matches >= column_positions.len() / 2
}

/// Mira hacia adelante para ver si hay un ":" después de un string (indicando que es una key JSON)
fn peek_for_colon(chars: &[char], start: usize) -> bool {
    let mut i = start;
    let mut in_string = true;

    while i < chars.len() {
        if in_string {
            if chars[i] == '"' && (i == 0 || chars[i - 1] != '\\') {
                in_string = false;
            }
        } else {
            if chars[i] == ':' {
                return true;
            }
            if !chars[i].is_whitespace() {
                return false;
            }
        }
        i += 1;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_log_levels() {
        assert_eq!(
            detect_log_level("INFO: Starting server"),
            Some(LogLevel::Info)
        );
        assert_eq!(
            detect_log_level("[ERROR] Failed to connect"),
            Some(LogLevel::Error)
        );
        assert_eq!(
            detect_log_level("WARN: Deprecated function"),
            Some(LogLevel::Warn)
        );
        assert_eq!(
            detect_log_level("DEBUG: Variable value = 42"),
            Some(LogLevel::Debug)
        );
    }

    #[test]
    fn test_detect_stack_traces() {
        assert!(is_stack_trace_line("at src/main.rs:42:5"));
        assert!(is_stack_trace_line("   at Main.main(Main.java:15)"));
        assert!(is_stack_trace_line(
            "  File \"script.py\", line 42, in function"
        ));
        assert!(is_stack_trace_line(
            "    at Object.<anonymous> (app.js:10:5)"
        ));
    }

    #[test]
    fn test_detect_errors() {
        assert!(is_error_line("error: could not compile"));
        assert!(is_error_line("Exception in thread main"));
        assert!(is_error_line("ERROR: Connection failed"));
    }

    #[test]
    fn test_detect_json() {
        assert!(is_json_line(r#"{"name": "John", "age": 30}"#));
        assert!(is_json_line(r#"  "config": {"debug": true}"#));
        assert!(is_json_line(r#"[1, 2, 3]"#));
    }

    #[test]
    fn test_detect_table_separator() {
        assert!(is_table_separator("+-------+-------+"));
        assert!(is_table_separator("|-------|-------|"));
        assert!(is_table_separator("===================="));
        assert!(!is_table_separator("normal text"));
    }
}

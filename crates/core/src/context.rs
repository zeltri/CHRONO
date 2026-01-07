use regex::Regex;
use std::sync::LazyLock;

/// Tipo de contexto semántico de una línea
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineContext {
    Normal,
    Error,
    Warning,
    StackTrace,
}

/// Información sobre un archivo detectado en la línea
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileReference {
    pub path: String,
    pub line: Option<usize>,
    pub col: Option<usize>,
    pub start_col: usize,  // Posición en la línea donde empieza la referencia
    pub end_col: usize,    // Posición donde termina
}

/// Patrones para detección de contexto
static ERROR_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)\berror\b").unwrap(),
        Regex::new(r"(?i)\bfailed\b").unwrap(),
        Regex::new(r"(?i)\bfatal\b").unwrap(),
        Regex::new(r"(?i)\bpanic\b").unwrap(),
        Regex::new(r"(?i)\bexception\b").unwrap(),
        Regex::new(r"(?i)\bcritical\b").unwrap(),
    ]
});

static WARNING_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)\bwarning\b").unwrap(),
        Regex::new(r"(?i)\bwarn\b").unwrap(),
        Regex::new(r"(?i)\bcaution\b").unwrap(),
        Regex::new(r"(?i)\bdeprecated\b").unwrap(),
    ]
});

// Patrones de archivo para diferentes formatos
static FILE_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // Rust: "at src/main.rs:42:10" o "src/main.rs:42:10"
        Regex::new(r"(?:at\s+)?([/\w\.-]+\.rs):(\d+):(\d+)").unwrap(),
        // Python: "File \"/path/to/file.py\", line 42"
        Regex::new(r#"File\s+"([^"]+\.py)",\s+line\s+(\d+)"#).unwrap(),
        // JavaScript: "at /path/to/file.js:42:10"
        Regex::new(r"at\s+([/\w\.-]+\.(?:js|ts)):(\d+):(\d+)").unwrap(),
        // C/C++: "/path/to/file.c:42:10"
        Regex::new(r"([/\w\.-]+\.(?:c|cpp|h|hpp)):(\d+):(\d+)").unwrap(),
        // Go: "/path/to/file.go:42:10"
        Regex::new(r"([/\w\.-]+\.go):(\d+):(\d+)").unwrap(),
        // General: cualquier archivo con línea
        Regex::new(r"([/\w\.-]+\.\w+):(\d+)(?::(\d+))?").unwrap(),
    ]
});

/// Analiza una línea y determina su contexto
pub fn analyze_line_context(text: &str) -> LineContext {
    // Primero verificar si es un stack trace (tiene referencia a archivo)
    if has_file_reference(text) {
        return LineContext::StackTrace;
    }
    
    // Luego verificar errores
    for pattern in ERROR_PATTERNS.iter() {
        if pattern.is_match(text) {
            return LineContext::Error;
        }
    }
    
    // Luego warnings
    for pattern in WARNING_PATTERNS.iter() {
        if pattern.is_match(text) {
            return LineContext::Warning;
        }
    }
    
    LineContext::Normal
}

/// Verifica si una línea contiene una referencia a archivo
pub fn has_file_reference(text: &str) -> bool {
    FILE_PATTERNS.iter().any(|p| p.is_match(text))
}

/// Extrae todas las referencias a archivos de una línea
pub fn extract_file_references(text: &str) -> Vec<FileReference> {
    let mut references = Vec::new();
    let mut seen_positions = std::collections::HashSet::new();
    
    for pattern in FILE_PATTERNS.iter() {
        for cap in pattern.captures_iter(text) {
            if let Some(path_match) = cap.get(1) {
                let start = path_match.start();
                
                // Evitar duplicados en la misma posición
                if seen_positions.contains(&start) {
                    continue;
                }
                seen_positions.insert(start);
                
                let path = path_match.as_str().to_string();
                let line = cap.get(2).and_then(|m| m.as_str().parse().ok());
                let col = cap.get(3).and_then(|m| m.as_str().parse().ok());
                
                references.push(FileReference {
                    path,
                    line,
                    col,
                    start_col: start,
                    end_col: cap.get(0).map(|m| m.end()).unwrap_or(path_match.end()),
                });
            }
        }
    }
    
    references
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_detection() {
        assert_eq!(analyze_line_context("Error: file not found"), LineContext::Error);
        assert_eq!(analyze_line_context("FATAL: connection failed"), LineContext::Error);
        assert_eq!(analyze_line_context("thread panicked at main.rs:42"), LineContext::StackTrace);
    }
    
    #[test]
    fn test_warning_detection() {
        assert_eq!(analyze_line_context("Warning: deprecated function"), LineContext::Warning);
        assert_eq!(analyze_line_context("WARN: low memory"), LineContext::Warning);
    }
    
    #[test]
    fn test_file_reference_rust() {
        let refs = extract_file_references("at src/main.rs:42:10");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].path, "src/main.rs");
        assert_eq!(refs[0].line, Some(42));
        assert_eq!(refs[0].col, Some(10));
    }
    
    #[test]
    fn test_file_reference_python() {
        let refs = extract_file_references(r#"File "/home/user/test.py", line 123"#);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].path, "/home/user/test.py");
        assert_eq!(refs[0].line, Some(123));
    }
    
    #[test]
    fn test_stack_trace_detection() {
        let line = "    at /home/user/project/src/lib.rs:156:9";
        assert_eq!(analyze_line_context(line), LineContext::StackTrace);
        assert!(has_file_reference(line));
    }
}

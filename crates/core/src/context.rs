use regex::Regex;
use std::sync::LazyLock;

/// Tipo de contexto semántico de una línea
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineContext {
    Normal,
    Error,
    Warning,
    StackTrace,
    FileList, // Línea con listado de archivos (ls, ll, etc.)
}

/// Tipo de archivo detectado para colorización
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileType {
    Directory,
    Executable,
    SymbolicLink,
    Archive,  // .tar, .gz, .zip, etc.
    Image,    // .png, .jpg, etc.
    Video,    // .mp4, .avi, etc.
    Audio,    // .mp3, .wav, etc.
    Document, // .pdf, .doc, etc.
    Code,     // .rs, .js, .py, etc.
    RegularFile,
}

/// Información sobre un archivo en el listado
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    pub name: String,
    pub file_type: FileType,
    pub is_executable: bool,
    pub start_col: usize,
    pub end_col: usize,
}

/// Información sobre un archivo detectado en la línea
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileReference {
    pub path: String,
    pub line: Option<usize>,
    pub col: Option<usize>,
    pub start_col: usize, // Posición en la línea donde empieza la referencia
    pub end_col: usize,   // Posición donde termina
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
    // Primero verificar si es un listado de archivos (detectar formato ls -l)
    if is_file_listing(text) {
        return LineContext::FileList;
    }

    // Verificar si es un stack trace (tiene referencia a archivo)
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

/// Detecta si una línea es un listado de archivos (formato ls -l o ls simple)
pub fn is_file_listing(text: &str) -> bool {
    let text = text.trim();
    if text.is_empty() {
        return false;
    }

    // Formato ls -l: permisos usuario grupo tamaño fecha nombre
    // Ejemplo: drwxr-xr-x 2 user group 4096 Jan 1 12:00 filename
    let ls_long_pattern = Regex::new(r"^[d\-lbcps][rwx\-]{9}\s+").unwrap();
    if ls_long_pattern.is_match(text) {
        return true;
    }

    // Formato simple: detectar si parece un nombre de archivo o directorio
    // Evitar false positives con líneas que son claramente output de comandos

    // Descartar líneas que claramente no son nombres de archivo
    if text.starts_with('$')
        || text.starts_with('>')
        || text.starts_with('#')
        || text.starts_with('[')
        || text.contains("  ") // múltiples espacios seguidos (probablemente output formateado)
        || text.len() > 100
    // nombres muy largos probablemente no son archivos
    {
        return false;
    }

    // Descartar mensajes de error/warning que contengan ':'
    if text.to_lowercase().contains("error")
        || text.to_lowercase().contains("warning")
        || text.to_lowercase().contains("failed")
    {
        return false;
    }

    // Aceptar si:
    // 1. Termina en / (directorio)
    // 2. Tiene extensión (archivo.ext)
    // 3. Empieza con . (archivo oculto)
    // 4. Solo contiene caracteres válidos para nombres de archivo
    let is_dir = text.ends_with('/');
    let has_extension = text.contains('.') && text.matches('.').count() <= 3; // max 3 puntos
    let is_hidden = text.starts_with('.');

    // Verificar que solo tenga caracteres válidos para nombres de archivo
    let valid_filename_chars = text
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-' || c == '/' || c == '~');

    if !valid_filename_chars {
        return false;
    }

    // Si pasa todas las verificaciones y tiene indicadores de archivo, aceptar
    is_dir || has_extension || is_hidden || (text.len() > 0 && text.len() < 50)
}

/// Extrae información de archivo desde una línea de ls -l
pub fn parse_file_entry(text: &str) -> Option<FileEntry> {
    // Patrón para ls -l: permisos links owner group size date time name
    let pattern = Regex::new(
        r"^([d\-lbcps])([rwx\-]{9})\s+\d+\s+\S+\s+\S+\s+\d+\s+\S+\s+\d+\s+[\d:]+\s+(.+)$",
    )
    .unwrap();

    if let Some(caps) = pattern.captures(text) {
        let type_char = caps.get(1)?.as_str();
        let perms = caps.get(2)?.as_str();
        let name = caps.get(3)?.as_str();

        let file_type = match type_char {
            "d" => FileType::Directory,
            "l" => FileType::SymbolicLink,
            _ => detect_file_type_by_extension(name),
        };

        // Verificar si tiene permisos de ejecución
        let is_executable = perms.chars().nth(2) == Some('x')
            || perms.chars().nth(5) == Some('x')
            || perms.chars().nth(8) == Some('x');

        // Encontrar posición del nombre en la línea
        let start_col = text.rfind(name)?;
        let end_col = start_col + name.len();

        Some(FileEntry {
            name: name.to_string(),
            file_type,
            is_executable,
            start_col,
            end_col,
        })
    } else {
        // Intentar parsear formato simple (solo nombres, como ls sin -l)
        parse_simple_file_entry(text)
    }
}

/// Parsea formato simple de listado (solo nombres)
fn parse_simple_file_entry(text: &str) -> Option<FileEntry> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Calcular el offset inicial (espacios antes del nombre)
    let start_col = text.len() - text.trim_start().len();

    // Detectar directorios por el sufijo /
    let (name, is_dir) = if trimmed.ends_with('/') {
        (trimmed.trim_end_matches('/'), true)
    } else {
        (trimmed, false)
    };

    // Si no tiene extensión y no termina en /, asumir que es un directorio
    // (comportamiento común de ls sin -F)
    let has_extension = name.contains('.');
    let likely_dir = !has_extension && !name.starts_with('.');

    let file_type = if is_dir || likely_dir {
        FileType::Directory
    } else {
        detect_file_type_by_extension(name)
    };

    // Detectar ejecutables por sufijos o extensiones conocidas
    // También considerar archivos sin extensión que podrían ser ejecutables
    let is_executable = name.ends_with(".sh")
        || name.ends_with(".exe")
        || name.ends_with(".bin")
        || name.ends_with(".out")
        || name.ends_with(".py")
        || name.ends_with(".rb")
        || name.ends_with(".pl")
        || (!has_extension && !is_dir && !likely_dir); // Archivo sin extensión

    Some(FileEntry {
        name: name.to_string(),
        file_type,
        is_executable,
        start_col,
        end_col: start_col + name.len(),
    })
}

/// Detecta tipo de archivo basado en su extensión
pub fn detect_file_type_by_extension(name: &str) -> FileType {
    let lower_name = name.to_lowercase();

    // Archivos comprimidos
    if lower_name.ends_with(".tar")
        || lower_name.ends_with(".gz")
        || lower_name.ends_with(".zip")
        || lower_name.ends_with(".7z")
        || lower_name.ends_with(".rar")
        || lower_name.ends_with(".bz2")
        || lower_name.ends_with(".xz")
    {
        return FileType::Archive;
    }

    // Imágenes
    if lower_name.ends_with(".png")
        || lower_name.ends_with(".jpg")
        || lower_name.ends_with(".jpeg")
        || lower_name.ends_with(".gif")
        || lower_name.ends_with(".bmp")
        || lower_name.ends_with(".svg")
        || lower_name.ends_with(".webp")
    {
        return FileType::Image;
    }

    // Videos
    if lower_name.ends_with(".mp4")
        || lower_name.ends_with(".avi")
        || lower_name.ends_with(".mkv")
        || lower_name.ends_with(".mov")
        || lower_name.ends_with(".wmv")
        || lower_name.ends_with(".flv")
        || lower_name.ends_with(".webm")
    {
        return FileType::Video;
    }

    // Audio
    if lower_name.ends_with(".mp3")
        || lower_name.ends_with(".wav")
        || lower_name.ends_with(".flac")
        || lower_name.ends_with(".aac")
        || lower_name.ends_with(".ogg")
        || lower_name.ends_with(".m4a")
    {
        return FileType::Audio;
    }

    // Documentos
    if lower_name.ends_with(".pdf")
        || lower_name.ends_with(".doc")
        || lower_name.ends_with(".docx")
        || lower_name.ends_with(".txt")
        || lower_name.ends_with(".md")
        || lower_name.ends_with(".odt")
    {
        return FileType::Document;
    }

    // Código
    if lower_name.ends_with(".rs")
        || lower_name.ends_with(".js")
        || lower_name.ends_with(".ts")
        || lower_name.ends_with(".py")
        || lower_name.ends_with(".c")
        || lower_name.ends_with(".cpp")
        || lower_name.ends_with(".h")
        || lower_name.ends_with(".hpp")
        || lower_name.ends_with(".go")
        || lower_name.ends_with(".java")
        || lower_name.ends_with(".rb")
        || lower_name.ends_with(".php")
        || lower_name.ends_with(".swift")
        || lower_name.ends_with(".kt")
    {
        return FileType::Code;
    }

    FileType::RegularFile
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_detection() {
        assert_eq!(
            analyze_line_context("Error: file not found"),
            LineContext::Error
        );
        assert_eq!(
            analyze_line_context("FATAL: connection failed"),
            LineContext::Error
        );
        assert_eq!(
            analyze_line_context("thread panicked at main.rs:42"),
            LineContext::StackTrace
        );
    }

    #[test]
    fn test_warning_detection() {
        assert_eq!(
            analyze_line_context("Warning: deprecated function"),
            LineContext::Warning
        );
        assert_eq!(
            analyze_line_context("WARN: low memory"),
            LineContext::Warning
        );
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

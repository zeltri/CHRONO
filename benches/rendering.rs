use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use terminal_core::{Cell, Screen};
use terminal_renderer::{CpuRenderer, ModernTheme};

fn bench_screen_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("screen_creation");

    for &(rows, cols) in &[(24, 80), (40, 120), (60, 200)] {
        group.bench_with_input(
            BenchmarkId::new("new_screen", format!("{}x{}", rows, cols)),
            &(rows, cols),
            |b, &(r, c)| {
                b.iter(|| {
                    let screen = Screen::new(black_box(r), black_box(c));
                    black_box(screen);
                });
            },
        );
    }

    group.finish();
}

fn bench_screen_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("screen_write");

    let mut screen = Screen::new(24, 80);

    group.bench_function("write_single_char", |b| {
        b.iter(|| {
            if let Some(cell) = screen.get_cell_mut(0, 0) {
                cell.set_char(black_box('A'));
            }
        });
    });

    group.bench_function("write_full_line", |b| {
        b.iter(|| {
            for col in 0..80 {
                if let Some(cell) = screen.get_cell_mut(0, col) {
                    cell.set_char(black_box('X'));
                }
            }
        });
    });

    group.bench_function("write_full_screen", |b| {
        b.iter(|| {
            for row in 0..24 {
                for col in 0..80 {
                    if let Some(cell) = screen.get_cell_mut(row, col) {
                        cell.set_char(black_box('█'));
                    }
                }
            }
        });
    });

    group.finish();
}

fn bench_renderer_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("renderer_creation");

    for &(width, height) in &[(800, 600), (1920, 1080), (2560, 1440)] {
        group.bench_with_input(
            BenchmarkId::new("new_renderer", format!("{}x{}", width, height)),
            &(width, height),
            |b, &(w, h)| {
                b.iter(|| {
                    let renderer = CpuRenderer::new(black_box(w), black_box(h));
                    black_box(renderer);
                });
            },
        );
    }

    group.finish();
}

fn bench_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("rendering");

    // Empty screen
    group.bench_function("render_empty_screen", |b| {
        let mut renderer = CpuRenderer::new(800, 600).unwrap();
        let screen = Screen::new(24, 80);

        b.iter(|| {
            renderer.render(black_box(&screen)).unwrap();
        });
    });

    // Screen with text
    group.bench_function("render_text_screen", |b| {
        let mut renderer = CpuRenderer::new(800, 600).unwrap();
        let mut screen = Screen::new(24, 80);

        // Fill with text
        for row in 0..24 {
            for col in 0..80 {
                if let Some(cell) = screen.get_cell_mut(row, col) {
                    cell.set_char('A');
                }
            }
        }

        b.iter(|| {
            renderer.render(black_box(&screen)).unwrap();
        });
    });

    // Screen with mixed content
    group.bench_function("render_mixed_screen", |b| {
        let mut renderer = CpuRenderer::new(800, 600).unwrap();
        let mut screen = Screen::new(24, 80);

        // Fill with varied characters
        let chars = ['A', 'B', 'C', '█', '▓', '░', '@', '#'];
        for row in 0..24 {
            for col in 0..80 {
                if let Some(cell) = screen.get_cell_mut(row, col) {
                    cell.set_char(chars[(row + col) % chars.len()]);
                }
            }
        }

        b.iter(|| {
            renderer.render(black_box(&screen)).unwrap();
        });
    });

    group.finish();
}

fn bench_theme_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("theme");

    group.bench_function("theme_creation", |b| {
        b.iter(|| {
            let theme = ModernTheme::new();
            black_box(theme);
        });
    });

    group.bench_function("ansi_color_lookup", |b| {
        let theme = ModernTheme::new();
        b.iter(|| {
            for i in 0..16 {
                let color = theme.ansi_color(black_box(i));
                black_box(color);
            }
        });
    });

    group.bench_function("rgb_conversion", |b| {
        b.iter(|| {
            for r in (0..=255).step_by(51) {
                for g in (0..=255).step_by(51) {
                    for b in (0..=255).step_by(51) {
                        let color =
                            ModernTheme::rgb_to_u32(black_box(r), black_box(g), black_box(b));
                        black_box(color);
                    }
                }
            }
        });
    });

    group.finish();
}

fn bench_context_detection(c: &mut Criterion) {
    use terminal_core::analyze_line_context;

    let mut group = c.benchmark_group("context_detection");

    let test_lines = vec![
        "normal text line",
        "Error: Something went wrong",
        "Warning: This is a warning",
        "drwxr-xr-x 2 user group 4096 Jan 1 file.txt",
        "    at main.rs:42:10",
        "ls -la",
    ];

    for (idx, line) in test_lines.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("analyze_line", idx), line, |b, &line| {
            b.iter(|| {
                let context = analyze_line_context(black_box(line));
                black_box(context);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_screen_creation,
    bench_screen_write,
    bench_renderer_creation,
    bench_rendering,
    bench_theme_operations,
    bench_context_detection,
);
criterion_main!(benches);

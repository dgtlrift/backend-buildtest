/// Build validation tests.
///
/// These tests generate code from reference schemas and attempt to build
/// the output using the real language toolchain.
///
/// They are marked `#[ignore]` so they don't run in CI by default.
/// Run them explicitly:
///
///   cargo test -p backend-buildtest --test build_validation -- --ignored --nocapture
///
/// Individual language:
///   cargo test -p backend-buildtest --test build_validation test_build_rust -- --ignored --nocapture

use backend_buildtest::{
    assert_all_ok, build_c, build_cpp, build_csharp, build_nodejs,
    build_python, build_rust, run_build_tests, BuildOutcome, SCHEMAS,
};
use backend_rust::RustBackend;
use backend_c::CBackend;
use backend_cpp::CppBackend;
use backend_csharp::CSharpBackend;
use backend_nodejs::NodeJsBackend;
use backend_python::PythonBackend;
use cddlc_codegen::CodegenOptions;

fn work_dir(lang: &str) -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix(&format!("cddlc-buildtest-{lang}-"))
        .tempdir()
        .expect("create temp dir")
}

// ── Schema generation smoke tests (always run, no toolchain needed) ───────────

#[test]
fn test_all_schemas_generate_rust() {
    let opts = CodegenOptions::default();
    for (name, src) in SCHEMAS {
        let files = backend_buildtest::generate_schema(name, src, &RustBackend, &opts)
            .unwrap_or_else(|e| panic!("schema '{name}' failed: {e}"));
        assert_eq!(files.len(), 3, "schema '{name}': expected 3 files");
    }
}

#[test]
fn test_all_schemas_generate_c() {
    let opts = CodegenOptions::default();
    for (name, src) in SCHEMAS {
        let files = backend_buildtest::generate_schema(name, src, &CBackend, &opts)
            .unwrap_or_else(|e| panic!("schema '{name}' failed: {e}"));
        assert_eq!(files.len(), 4, "schema '{name}': expected 4 files");
    }
}

#[test]
fn test_all_schemas_generate_cpp() {
    let opts = CodegenOptions::default();
    for (name, src) in SCHEMAS {
        let files = backend_buildtest::generate_schema(name, src, &CppBackend, &opts)
            .unwrap_or_else(|e| panic!("schema '{name}' failed: {e}"));
        assert_eq!(files.len(), 4, "schema '{name}': expected 4 files");
    }
}

#[test]
fn test_all_schemas_generate_csharp() {
    let opts = CodegenOptions::default();
    for (name, src) in SCHEMAS {
        let files = backend_buildtest::generate_schema(name, src, &CSharpBackend, &opts)
            .unwrap_or_else(|e| panic!("schema '{name}' failed: {e}"));
        assert_eq!(files.len(), 3, "schema '{name}': expected 3 files");
    }
}

#[test]
fn test_all_schemas_generate_nodejs() {
    let opts = CodegenOptions::default();
    for (name, src) in SCHEMAS {
        let files = backend_buildtest::generate_schema(name, src, &NodeJsBackend, &opts)
            .unwrap_or_else(|e| panic!("schema '{name}' failed: {e}"));
        assert_eq!(files.len(), 4, "schema '{name}': expected 4 files");
    }
}

#[test]
fn test_all_schemas_generate_python() {
    let opts = CodegenOptions::default();
    for (name, src) in SCHEMAS {
        let files = backend_buildtest::generate_schema(name, src, &PythonBackend, &opts)
            .unwrap_or_else(|e| panic!("schema '{name}' failed: {e}"));
        assert_eq!(files.len(), 3, "schema '{name}': expected 3 files");
    }
}

// ── Content spot checks (always run) ─────────────────────────────────────────

#[test]
fn test_iot_sensor_rust_types() {
    let src  = SCHEMAS.iter().find(|(n, _)| *n == "iot-sensor").unwrap().1;
    let files = backend_buildtest::generate_schema("iot-sensor", src, &RustBackend, &CodegenOptions::default()).unwrap();
    let lib_rs = files.iter().find(|f| f.path.ends_with("lib.rs")).unwrap();
    assert!(lib_rs.content.contains("pub struct SensorMessage"), "missing SensorMessage");
    assert!(lib_rs.content.contains("pub struct SensorReading"),  "missing SensorReading");
    assert!(lib_rs.content.contains("pub enum SensorType"),       "missing SensorType");
}

#[test]
fn test_iot_sensor_python_types() {
    let src  = SCHEMAS.iter().find(|(n, _)| *n == "iot-sensor").unwrap().1;
    let files = backend_buildtest::generate_schema("iot-sensor", src, &PythonBackend, &CodegenOptions::default()).unwrap();
    let py = files.iter().find(|f| f.path.ends_with(".py")).unwrap();
    assert!(py.content.contains("class SensorMessage:"), "missing SensorMessage");
    assert!(py.content.contains("class SensorReading:"),  "missing SensorReading");
    assert!(py.content.contains("class SensorType:"),     "missing SensorType");
}

#[test]
fn test_iot_sensor_csharp_types() {
    let src  = SCHEMAS.iter().find(|(n, _)| *n == "iot-sensor").unwrap().1;
    let files = backend_buildtest::generate_schema("iot-sensor", src, &CSharpBackend, &CodegenOptions::default()).unwrap();
    let cs = files.iter().find(|f| f.path.ends_with(".cs")).unwrap();
    assert!(cs.content.contains("class SensorMessage"), "missing SensorMessage");
    assert!(cs.content.contains("class SensorReading"),  "missing SensorReading");
    assert!(cs.content.contains("class SensorType"),     "missing SensorType");
}

#[test]
fn test_iot_sensor_nodejs_types() {
    let src  = SCHEMAS.iter().find(|(n, _)| *n == "iot-sensor").unwrap().1;
    let files = backend_buildtest::generate_schema("iot-sensor", src, &NodeJsBackend, &CodegenOptions::default()).unwrap();
    let ts = files.iter().find(|f| f.path.ends_with(".ts") && f.path.contains("/src/")).unwrap();
    assert!(ts.content.contains("SensorMessage"), "missing SensorMessage");
    assert!(ts.content.contains("SensorReading"),  "missing SensorReading");
    assert!(ts.content.contains("SensorType"),     "missing SensorType");
}

#[test]
fn test_dcbor_generates_for_all_schemas() {
    let mut opts = CodegenOptions::default();
    opts.dcbor = true;
    for (name, src) in SCHEMAS {
        let _ = backend_buildtest::generate_schema(name, src, &RustBackend, &opts)
            .unwrap_or_else(|e| panic!("dcbor schema '{name}' failed: {e}"));
    }
}

// ── Actual build tests (ignored by default, require installed toolchain) ──────

/// Build all schemas using the Rust backend.
/// Requires: cargo in PATH
/// Run: cargo test -p backend-buildtest --test build_validation test_build_rust -- --ignored --nocapture
#[test]
#[ignore]
fn test_build_rust() {
    let tmp  = work_dir("rust");
    let opts = CodegenOptions::default();
    eprintln!("\n=== Rust build test ===");
    let outcomes = run_build_tests(&RustBackend, |d, f, n| build_rust(d, f, n), &opts, tmp.path());
    print_summary("Rust", &outcomes);
    assert_all_ok(&outcomes);
}

/// Build all schemas using the C backend.
/// Requires: cmake, cc in PATH
/// Run: cargo test -p backend-buildtest --test build_validation test_build_c -- --ignored --nocapture
#[test]
#[ignore]
fn test_build_c() {
    let tmp  = work_dir("c");
    let opts = CodegenOptions::default();
    eprintln!("\n=== C build test ===");
    let outcomes = run_build_tests(&CBackend, |d, f, n| build_c(d, f, n), &opts, tmp.path());
    print_summary("C", &outcomes);
    assert_all_ok(&outcomes);
}

/// Build all schemas using the C++ backend.
/// Requires: cmake, c++ in PATH
/// Run: cargo test -p backend-buildtest --test build_validation test_build_cpp -- --ignored --nocapture
#[test]
#[ignore]
fn test_build_cpp() {
    let tmp  = work_dir("cpp");
    let opts = CodegenOptions::default();
    eprintln!("\n=== C++ build test ===");
    let outcomes = run_build_tests(&CppBackend, |d, f, n| build_cpp(d, f, n), &opts, tmp.path());
    print_summary("C++", &outcomes);
    assert_all_ok(&outcomes);
}

/// Build all schemas using the C# backend.
/// Requires: dotnet in PATH
/// Run: cargo test -p backend-buildtest --test build_validation test_build_csharp -- --ignored --nocapture
#[test]
#[ignore]
fn test_build_csharp() {
    let tmp  = work_dir("csharp");
    let opts = CodegenOptions::default();
    eprintln!("\n=== C# build test ===");
    let outcomes = run_build_tests(&CSharpBackend, |d, f, n| build_csharp(d, f, n), &opts, tmp.path());
    print_summary("C#", &outcomes);
    assert_all_ok(&outcomes);
}

/// Build all schemas using the Node.js backend.
/// Requires: npm in PATH
/// Run: cargo test -p backend-buildtest --test build_validation test_build_nodejs -- --ignored --nocapture
#[test]
#[ignore]
fn test_build_nodejs() {
    let tmp  = work_dir("nodejs");
    let opts = CodegenOptions::default();
    eprintln!("\n=== Node.js build test ===");
    let outcomes = run_build_tests(&NodeJsBackend, |d, f, n| build_nodejs(d, f, n), &opts, tmp.path());
    print_summary("Node.js", &outcomes);
    assert_all_ok(&outcomes);
}

/// Build all schemas using the Python backend.
/// Requires: python3 in PATH
/// Run: cargo test -p backend-buildtest --test build_validation test_build_python -- --ignored --nocapture
#[test]
#[ignore]
fn test_build_python() {
    let tmp  = work_dir("python");
    let opts = CodegenOptions::default();
    eprintln!("\n=== Python build test ===");
    let outcomes = run_build_tests(&PythonBackend, |d, f, n| build_python(d, f, n), &opts, tmp.path());
    print_summary("Python", &outcomes);
    assert_all_ok(&outcomes);
}

/// Run all language build tests in sequence with a final summary table.
/// Run: cargo test -p backend-buildtest --test build_validation test_build_all -- --ignored --nocapture
#[test]
#[ignore]
fn test_build_all() {
    let opts = CodegenOptions::default();

    eprintln!("\n╔══════════════════════════════════════╗");
    eprintln!("║   cddlc backend build validation    ║");
    eprintln!("╚══════════════════════════════════════╝");

    let mut all_outcomes: Vec<(&str, Vec<BuildOutcome>)> = Vec::new();

    macro_rules! run_lang {
        ($label:expr, $backend:expr, $build_fn:expr, $suffix:expr) => {{
            eprintln!("\n─── {} ───────────────────────────", $label);
            let tmp = work_dir($suffix);
            let outcomes = run_build_tests(&$backend, $build_fn, &opts, tmp.path());
            print_summary($label, &outcomes);
            all_outcomes.push(($label, outcomes));
        }};
    }

    run_lang!("Rust",    RustBackend,   |d, f, n| build_rust(d, f, n),    "rust");
    run_lang!("C",       CBackend,      |d, f, n| build_c(d, f, n),       "c");
    run_lang!("C++",     CppBackend,    |d, f, n| build_cpp(d, f, n),     "cpp");
    run_lang!("C#",      CSharpBackend, |d, f, n| build_csharp(d, f, n),  "csharp");
    run_lang!("Node.js", NodeJsBackend, |d, f, n| build_nodejs(d, f, n),  "nodejs");
    run_lang!("Python",  PythonBackend, |d, f, n| build_python(d, f, n),  "python");

    eprintln!("\n╔════════════╦═══════╦═══════╦═══════╗");
    eprintln!("║ Language   ║  OK   ║ SKIP  ║ FAIL  ║");
    eprintln!("╠════════════╬═══════╬═══════╬═══════╣");
    for (lang, outcomes) in &all_outcomes {
        let ok   = outcomes.iter().filter(|o| matches!(o, BuildOutcome::Built { .. })).count();
        let skip = outcomes.iter().filter(|o| o.is_skipped()).count();
        let fail = outcomes.iter().filter(|o| !o.is_ok()).count();
        eprintln!("║ {lang:<10} ║  {ok:<4} ║  {skip:<4} ║  {fail:<4} ║");
    }
    eprintln!("╚════════════╩═══════╩═══════╩═══════╝");

    for (_lang, outcomes) in &all_outcomes {
        assert_all_ok(outcomes);
    }
}

fn print_summary(lang: &str, outcomes: &[BuildOutcome]) {
    let ok   = outcomes.iter().filter(|o| matches!(o, BuildOutcome::Built { .. })).count();
    let skip = outcomes.iter().filter(|o| o.is_skipped()).count();
    let fail = outcomes.iter().filter(|o| !o.is_ok()).count();
    eprintln!("  {lang}: {ok} built, {skip} skipped, {fail} failed / {} schemas", outcomes.len());
    if let Some(BuildOutcome::Skipped { reason }) = outcomes.first() {
        eprintln!("  (skip reason: {reason})");
    }
}

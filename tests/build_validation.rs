/// Build validation tests.
///
/// These tests generate code from reference schemas and attempt to build
/// the output using the real language toolchain.
///
/// They are marked `#[ignore]` so they don't run in CI by default.
/// Run them explicitly:
///
///   cargo test -p backend-buildtest --test build_validation -- --ignored
///   cargo test -p backend-buildtest --test build_validation rust -- --ignored
///   CDDLC_BUILD_TESTS=1 cargo test -p backend-buildtest --test build_validation -- --ignored
///
/// Individual language:
///   cargo test -p backend-buildtest --test build_validation build_rust -- --ignored --nocapture

use std::path::PathBuf;

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

// ── Temp dir helper ───────────────────────────────────────────────────────────

fn work_dir(lang: &str) -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix(&format!("cddlc-buildtest-{lang}-"))
        .tempdir()
        .expect("create temp dir")
}

// ── Schema generation smoke tests (no build, always run) ─────────────────────

#[test]
fn test_all_schemas_parse_and_generate_rust() {
    let opts = CodegenOptions::default();
    for (name, src) in SCHEMAS {
        let files = backend_buildtest::generate_schema(name, src, &RustBackend, &opts)
            .unwrap_or_else(|e| panic!("schema '{name}' failed: {e}"));
        assert_eq!(files.len(), 3, "schema '{name}': expected 3 files (Cargo.toml, lib.rs, roundtrip.rs)");
    }
}

#[test]
fn test_all_schemas_parse_and_generate_c() {
    let opts = CodegenOptions::default();
    for (name, src) in SCHEMAS {
        let files = backend_buildtest::generate_schema(name, src, &CBackend, &opts)
            .unwrap_or_else(|e| panic!("schema '{name}' failed: {e}"));
        assert_eq!(files.len(), 4, "schema '{name}': expected 4 files");
    }
}

#[test]
fn test_all_schemas_parse_and_generate_cpp() {
    let opts = CodegenOptions::default();
    for (name, src) in SCHEMAS {
        let files = backend_buildtest::generate_schema(name, src, &CppBackend, &opts)
            .unwrap_or_else(|e| panic!("schema '{name}' failed: {e}"));
        assert_eq!(files.len(), 4, "schema '{name}': expected 4 files");
    }
}

#[test]
fn test_all_schemas_parse_and_generate_csharp() {
    let opts = CodegenOptions::default();
    for (name, src) in SCHEMAS {
        let files = backend_buildtest::generate_schema(name, src, &CSharpBackend, &opts)
            .unwrap_or_else(|e| panic!("schema '{name}' failed: {e}"));
        assert_eq!(files.len(), 3, "schema '{name}': expected 3 files");
    }
}

#[test]
fn test_all_schemas_parse_and_generate_nodejs() {
    let opts = CodegenOptions::default();
    for (name, src) in SCHEMAS {
        let files = backend_buildtest::generate_schema(name, src, &NodeJsBackend, &opts)
            .unwrap_or_else(|e| panic!("schema '{name}' failed: {e}"));
        assert_eq!(files.len(), 4, "schema '{name}': expected 4 files");
    }
}

#[test]
fn test_all_schemas_parse_and_generate_python() {
    let opts = CodegenOptions::default();
    for (name, src) in SCHEMAS {
        let files = backend_buildtest::generate_schema(name, src, &PythonBackend, &opts)
            .unwrap_or_else(|e| panic!("schema '{name}' failed: {e}"));
        assert_eq!(files.len(), 3, "schema '{name}': expected 3 files");
    }
}

// ── Content spot checks (always run) ─────────────────────────────────────────

#[test]
fn test_iot_sensor_rust_contains_sensor_message() {
    let src  = backend_buildtest::SCHEMAS.iter().find(|(n, _)| *n == "iot-sensor").unwrap().1;
    let opts = CodegenOptions::default();
    let files = backend_buildtest::generate_schema("iot-sensor", src, &RustBackend, &opts).unwrap();
    let lib_rs = files.iter().find(|f| f.path.ends_with("lib.rs")).unwrap();
    assert!(lib_rs.content.contains("pub struct SensorMessage"), "expected SensorMessage struct");
    assert!(lib_rs.content.contains("pub struct SensorReading"),  "expected SensorReading struct");
    assert!(lib_rs.content.contains("pub enum SensorType"),       "expected SensorType enum");
}

#[test]
fn test_iot_sensor_python_contains_classes() {
    let src  = backend_buildtest::SCHEMAS.iter().find(|(n, _)| *n == "iot-sensor").unwrap().1;
    let opts = CodegenOptions::default();
    let files = backend_buildtest::generate_schema("iot-sensor", src, &PythonBackend, &opts).unwrap();
    let py = files.iter().find(|f| f.path.ends_with(".py")).unwrap();
    assert!(py.content.contains("class SensorMessage:"));
    assert!(py.content.contains("class SensorReading:"));
    assert!(py.content.contains("class SensorType:"));
}

#[test]
fn test_iot_sensor_csharp_contains_classes() {
    let src  = backend_buildtest::SCHEMAS.iter().find(|(n, _)| *n == "iot-sensor").unwrap().1;
    let opts = CodegenOptions::default();
    let files = backend_buildtest::generate_schema("iot-sensor", src, &CSharpBackend, &opts).unwrap();
    let cs = files.iter().find(|f| f.path.ends_with(".cs")).unwrap();
    assert!(cs.content.contains("class SensorMessage"));
    assert!(cs.content.contains("class SensorReading"));
    assert!(cs.content.contains("class SensorType"));
}

#[test]
fn test_iot_sensor_nodejs_contains_interfaces() {
    let src  = backend_buildtest::SCHEMAS.iter().find(|(n, _)| *n == "iot-sensor").unwrap().1;
    let opts = CodegenOptions::default();
    let files = backend_buildtest::generate_schema("iot-sensor", src, &NodeJsBackend, &opts).unwrap();
    let ts = files.iter().find(|f| f.path.ends_with(".ts") && f.path.contains("/src/")).unwrap();
    assert!(ts.content.contains("interface SensorMessage") || ts.content.contains("SensorMessage"));
    assert!(ts.content.contains("SensorReading"));
    assert!(ts.content.contains("SensorType"));
}

#[test]
fn test_dcbor_option_generates_for_all_schemas() {
    let mut opts = CodegenOptions::default();
    opts.dcbor = true;
    for (name, src) in SCHEMAS {
        // Should not panic for any schema with dCBOR enabled
        let _ = backend_buildtest::generate_schema(name, src, &RustBackend, &opts)
            .unwrap_or_else(|e| panic!("dcbor schema '{name}' failed: {e}"));
    }
}

// ── Actual build tests (ignored by default, require toolchain) ────────────────

/// Build all reference schemas using the Rust backend.
///
/// Requires: cargo in PATH
/// Run: cargo test -p backend-buildtest --test build_validation build_rust -- --ignored --nocapture
#[test]
#[ignore]
fn build_rust() {
    let tmp  = work_dir("rust");
    let opts = CodegenOptions::default();
    eprintln!("\n=== Rust build test ===");
    let outcomes = run_build_tests(&RustBackend, |dir, files, name| {
        build_rust(dir, files, name)
    }, &opts, tmp.path());
    print_summary("Rust", &outcomes);
    assert_all_ok(&outcomes);
}

/// Build all reference schemas using the C backend.
///
/// Requires: cmake, cc in PATH, internet for NanoCBOR FetchContent
/// Run: cargo test -p backend-buildtest --test build_validation build_c -- --ignored --nocapture
#[test]
#[ignore]
fn build_c() {
    let tmp  = work_dir("c");
    let opts = CodegenOptions::default();
    eprintln!("\n=== C build test ===");
    let outcomes = run_build_tests(&CBackend, |dir, files, name| {
        build_c(dir, files, name)
    }, &opts, tmp.path());
    print_summary("C", &outcomes);
    assert_all_ok(&outcomes);
}

/// Build all reference schemas using the C++ backend.
///
/// Requires: cmake, c++ in PATH, internet for NanoCBOR FetchContent
/// Run: cargo test -p backend-buildtest --test build_validation build_cpp -- --ignored --nocapture
#[test]
#[ignore]
fn build_cpp() {
    let tmp  = work_dir("cpp");
    let opts = CodegenOptions::default();
    eprintln!("\n=== C++ build test ===");
    let outcomes = run_build_tests(&CppBackend, |dir, files, name| {
        build_cpp(dir, files, name)
    }, &opts, tmp.path());
    print_summary("C++", &outcomes);
    assert_all_ok(&outcomes);
}

/// Build all reference schemas using the C# backend.
///
/// Requires: dotnet in PATH
/// Run: cargo test -p backend-buildtest --test build_validation build_csharp -- --ignored --nocapture
#[test]
#[ignore]
fn build_csharp() {
    let tmp  = work_dir("csharp");
    let opts = CodegenOptions::default();
    eprintln!("\n=== C# build test ===");
    let outcomes = run_build_tests(&CSharpBackend, |dir, files, name| {
        build_csharp(dir, files, name)
    }, &opts, tmp.path());
    print_summary("C#", &outcomes);
    assert_all_ok(&outcomes);
}

/// Build all reference schemas using the Node.js backend.
///
/// Requires: npm, node in PATH, internet for npm install
/// Run: cargo test -p backend-buildtest --test build_validation build_nodejs -- --ignored --nocapture
#[test]
#[ignore]
fn build_nodejs() {
    let tmp  = work_dir("nodejs");
    let opts = CodegenOptions::default();
    eprintln!("\n=== Node.js build test ===");
    let outcomes = run_build_tests(&NodeJsBackend, |dir, files, name| {
        build_nodejs(dir, files, name)
    }, &opts, tmp.path());
    print_summary("Node.js", &outcomes);
    assert_all_ok(&outcomes);
}

/// Build all reference schemas using the Python backend.
///
/// Requires: python3 in PATH; cbor2 installed for import checks
/// Run: cargo test -p backend-buildtest --test build_validation build_python -- --ignored --nocapture
#[test]
#[ignore]
fn build_python() {
    let tmp  = work_dir("python");
    let opts = CodegenOptions::default();
    eprintln!("\n=== Python build test ===");
    let outcomes = run_build_tests(&PythonBackend, |dir, files, name| {
        build_python(dir, files, name)
    }, &opts, tmp.path());
    print_summary("Python", &outcomes);
    assert_all_ok(&outcomes);
}

/// Run all language build tests in sequence.
///
/// Run: cargo test -p backend-buildtest --test build_validation build_all -- --ignored --nocapture
#[test]
#[ignore]
fn build_all() {
    let opts = CodegenOptions::default();

    let languages: &[(&str, Box<dyn Fn() -> Vec<BuildOutcome>>)] = &[
        ("Rust",    Box::new(|| { let tmp = work_dir("rust");    run_build_tests(&RustBackend,   |d, f, n| build_rust(d, f, n),    &opts, tmp.path()) })),
        ("C",       Box::new(|| { let tmp = work_dir("c");       run_build_tests(&CBackend,      |d, f, n| build_c(d, f, n),       &opts, tmp.path()) })),
        ("C++",     Box::new(|| { let tmp = work_dir("cpp");     run_build_tests(&CppBackend,    |d, f, n| build_cpp(d, f, n),     &opts, tmp.path()) })),
        ("C#",      Box::new(|| { let tmp = work_dir("csharp");  run_build_tests(&CSharpBackend, |d, f, n| build_csharp(d, f, n),  &opts, tmp.path()) })),
        ("Node.js", Box::new(|| { let tmp = work_dir("nodejs");  run_build_tests(&NodeJsBackend, |d, f, n| build_nodejs(d, f, n),  &opts, tmp.path()) })),
        ("Python",  Box::new(|| { let tmp = work_dir("python");  run_build_tests(&PythonBackend, |d, f, n| build_python(d, f, n),  &opts, tmp.path()) })),
    ];

    eprintln!("\n╔══════════════════════════════════════╗");
    eprintln!("║   cddlc backend build validation    ║");
    eprintln!("╚══════════════════════════════════════╝");

    let mut all_outcomes: Vec<(&str, Vec<BuildOutcome>)> = Vec::new();
    for (lang, run) in languages {
        eprintln!("\n─── {lang} ───────────────────────────");
        let outcomes = run();
        print_summary(lang, &outcomes);
        all_outcomes.push((lang, outcomes));
    }

    // Final summary table
    eprintln!("\n╔══════════════════════════════════════╗");
    eprintln!("║            Final Summary             ║");
    eprintln!("╠═════════╦═══════╦═══════╦═══════════╣");
    eprintln!("║ Language ║  OK   ║ SKIP  ║  FAIL     ║");
    eprintln!("╠═════════╬═══════╬═══════╬═══════════╣");
    for (lang, outcomes) in &all_outcomes {
        let ok   = outcomes.iter().filter(|o| matches!(o, BuildOutcome::Built { .. })).count();
        let skip = outcomes.iter().filter(|o| o.is_skipped()).count();
        let fail = outcomes.iter().filter(|o| !o.is_ok()).count();
        eprintln!("║ {lang:<8} ║  {ok:<4} ║  {skip:<4} ║  {fail:<8} ║");
    }
    eprintln!("╚══════════╩═══════╩═══════╩═══════════╝");

    // Fail if any non-skipped outcome failed
    for (lang, outcomes) in &all_outcomes {
        let failures: Vec<_> = outcomes.iter().filter(|o| !o.is_ok()).collect();
        if !failures.is_empty() {
            eprintln!("\n{lang} had {} failure(s)", failures.len());
            assert_all_ok(outcomes);
        }
    }
}

// ── Summary printer ───────────────────────────────────────────────────────────

fn print_summary(lang: &str, outcomes: &[BuildOutcome]) {
    let ok   = outcomes.iter().filter(|o| matches!(o, BuildOutcome::Built { .. })).count();
    let skip = outcomes.iter().filter(|o| o.is_skipped()).count();
    let fail = outcomes.iter().filter(|o| !o.is_ok()).count();
    eprintln!("\n  {lang} summary: {ok} built, {skip} skipped, {fail} failed / {} schemas",
        outcomes.len());
    if let Some(BuildOutcome::Skipped { reason }) = outcomes.first() {
        eprintln!("  (skip reason: {reason})");
    }
}

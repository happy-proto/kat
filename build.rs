use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    process,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use blake3::Hasher;
use cc::Build;
use serde::Deserialize;
use shadow_rs::{BuildPattern, ShadowBuilder};
use tree_sitter_generate::{ABI_VERSION_MIN, OptLevel, generate_parser_in_directory};
use walkdir::WalkDir;

const PARSER_OPT_LEVEL_NAME: &str = "default";
const TREE_SITTER_CACHE_VERSION: &str = "v1";
const TREE_SITTER_CACHE_ROOT_DIR: &str = ".build-cache";

#[derive(Debug, Deserialize)]
struct GrammarRegistry {
    grammar: Vec<GrammarSpec>,
}

#[derive(Clone, Debug, Deserialize)]
struct GrammarSpec {
    name: String,
    library_name: String,
    #[serde(default)]
    parser_source: ParserSource,
    extra_c_flags: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum ParserSource {
    #[default]
    Vendored,
    Crate,
}

struct GrammarCachePaths {
    grammar_json_dir: PathBuf,
    grammar_json_staging_dir: PathBuf,
    grammar_json_fingerprint_path: PathBuf,
    parser_src_dir: PathBuf,
    parser_src_staging_dir: PathBuf,
    parser_fingerprint_path: PathBuf,
    native_out_dir: PathBuf,
    native_fingerprint_path: PathBuf,
}

#[derive(Clone)]
struct BuildContext {
    manifest_dir: PathBuf,
    shared_cache_root: PathBuf,
    build_target: String,
    build_profile: String,
}

struct BuildProfiler {
    enabled: bool,
    log_path: Option<PathBuf>,
}

struct GrammarBuildResult {
    native_out_dir: PathBuf,
    library_name: String,
    cpp_library_name: Option<String>,
}

fn main() {
    ShadowBuilder::builder()
        .build_pattern(BuildPattern::RealTime)
        .build()
        .expect("failed to generate shadow build metadata");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("missing manifest dir"));
    let build_target = env::var("TARGET").expect("missing TARGET");
    let build_profile = env::var("PROFILE").expect("missing PROFILE");
    let registry_path = manifest_dir.join("grammars/registry.toml");
    let shared_cache_root = manifest_dir
        .join(TREE_SITTER_CACHE_ROOT_DIR)
        .join("tree-sitter-cache")
        .join(TREE_SITTER_CACHE_VERSION);
    let profiler = BuildProfiler::from_env();
    let process_staging_root = shared_cache_root
        .join("staging")
        .join(format!("pid-{}", process::id()));
    let build_context = BuildContext {
        manifest_dir: manifest_dir.clone(),
        shared_cache_root: shared_cache_root.clone(),
        build_target: build_target.clone(),
        build_profile: build_profile.clone(),
    };
    let registry: GrammarRegistry = toml::from_str(
        &fs::read_to_string(&registry_path).expect("failed to read grammars/registry.toml"),
    )
    .expect("failed to parse grammars/registry.toml");

    println!("cargo:rerun-if-changed={}", registry_path.display());
    remove_dir_if_exists(&process_staging_root);
    profiler.log_global(format!(
        "shared cache root={} target={} profile={}",
        shared_cache_root.display(),
        build_target,
        build_profile
    ));

    for grammar in &registry.grammar {
        let grammar_dir = manifest_dir.join("grammars").join(&grammar.name);
        emit_grammar_source_watch_directives(&grammar_dir);
    }

    let grammar_specs = Arc::new(registry.grammar);
    let next_index = Arc::new(AtomicUsize::new(0));
    let job_count = grammar_job_count(grammar_specs.len());
    profiler.log_global(format!("grammar_jobs={job_count}"));

    let mut build_results = Vec::new();
    thread::scope(|scope| {
        let mut handles = Vec::new();

        for _ in 0..job_count {
            let grammar_specs = Arc::clone(&grammar_specs);
            let next_index = Arc::clone(&next_index);
            let build_context = build_context.clone();
            let profiler = &profiler;

            handles.push(scope.spawn(move || {
                let mut worker_results = Vec::new();

                loop {
                    let index = next_index.fetch_add(1, Ordering::Relaxed);
                    let Some(grammar) = grammar_specs.get(index) else {
                        break;
                    };

                    if let Some(build_result) = compile_grammar(&build_context, grammar, profiler) {
                        worker_results.push(build_result);
                    }
                }

                worker_results
            }));
        }

        for handle in handles {
            build_results.extend(
                handle
                    .join()
                    .unwrap_or_else(|payload| std::panic::resume_unwind(payload)),
            );
        }
    });

    for build_result in build_results {
        println!(
            "cargo:rustc-link-search=native={}",
            build_result.native_out_dir.display()
        );
        emit_link_directive(&build_result.library_name);
        if let Some(cpp_library_name) = &build_result.cpp_library_name {
            emit_link_directive(cpp_library_name);
            emit_cpp_runtime_link_directive();
        }
    }

    remove_dir_if_exists(&process_staging_root);
}

fn compile_grammar(
    build_context: &BuildContext,
    grammar: &GrammarSpec,
    profiler: &BuildProfiler,
) -> Option<GrammarBuildResult> {
    if grammar.parser_source == ParserSource::Crate {
        profiler.log_global(format!(
            "grammar={} source=crate skip-local-build",
            grammar.name
        ));
        return None;
    }

    let grammar_started_at = Instant::now();
    let grammar_dir = build_context
        .manifest_dir
        .join("grammars")
        .join(&grammar.name);
    let cache_paths = GrammarCachePaths::new(
        &build_context.shared_cache_root,
        &grammar.name,
        &build_context.build_target,
        &build_context.build_profile,
    );

    let grammar_json_input_paths = collect_support_paths(&grammar_dir, &["js", "json"]);
    let c_scanner_paths = collect_scanner_paths(&grammar_dir, &["scanner.c"]);
    let cpp_scanner_paths = collect_scanner_paths(&grammar_dir, &["scanner.cc", "scanner.cpp"]);
    let native_support_paths = collect_support_paths(&grammar_dir, &["h", "hh", "hpp"]);

    let grammar_json_path = cache_paths.grammar_json_dir.join("grammar.json");
    let grammar_json_fingerprint = grammar_json_generation_fingerprint(&grammar_json_input_paths);
    if !fingerprint_matches(
        &cache_paths.grammar_json_fingerprint_path,
        &grammar_json_fingerprint,
    ) || !grammar_json_path.exists()
    {
        let stage_started_at = Instant::now();
        regenerate_grammar_json(&grammar_dir, &cache_paths);
        write_fingerprint(
            &cache_paths.grammar_json_fingerprint_path,
            &grammar_json_fingerprint,
        );
        profiler.log_stage(
            &grammar.name,
            "grammar-json",
            "rebuild",
            stage_started_at.elapsed(),
        );
    } else {
        profiler.log_stage(&grammar.name, "grammar-json", "cache-hit", Duration::ZERO);
    }

    let parser_fingerprint = parser_generation_fingerprint(&grammar_json_path);
    let cpp_library_name = format!("{}-cpp-scanner", grammar.library_name);

    if !fingerprint_matches(&cache_paths.parser_fingerprint_path, &parser_fingerprint)
        || !parser_artifacts_exist(&cache_paths.parser_src_dir)
    {
        let stage_started_at = Instant::now();
        regenerate_parser_sources(&grammar_dir, &grammar_json_path, &cache_paths);
        write_fingerprint(&cache_paths.parser_fingerprint_path, &parser_fingerprint);
        profiler.log_stage(
            &grammar.name,
            "parser-src",
            "rebuild",
            stage_started_at.elapsed(),
        );
    } else {
        profiler.log_stage(&grammar.name, "parser-src", "cache-hit", Duration::ZERO);
    }

    let native_fingerprint = native_compile_fingerprint(
        grammar,
        &cache_paths.parser_src_dir.join("parser.c"),
        &c_scanner_paths,
        &cpp_scanner_paths,
        &native_support_paths,
        &grammar_dir,
        &cache_paths.parser_src_dir,
    );

    if !fingerprint_matches(&cache_paths.native_fingerprint_path, &native_fingerprint)
        || !library_archive_path(&cache_paths.native_out_dir, &grammar.library_name).exists()
        || (!cpp_scanner_paths.is_empty()
            && !library_archive_path(&cache_paths.native_out_dir, &cpp_library_name).exists())
    {
        let stage_started_at = Instant::now();
        compile_c_sources(
            &grammar_dir,
            &cache_paths.parser_src_dir,
            &cache_paths.native_out_dir,
            &grammar.library_name,
            &grammar.extra_c_flags,
            {
                let mut paths = vec![cache_paths.parser_src_dir.join("parser.c")];
                paths.extend(c_scanner_paths.clone());
                paths
            },
        );

        if !cpp_scanner_paths.is_empty() {
            compile_cpp_sources(
                &grammar_dir,
                &cache_paths.parser_src_dir,
                &cache_paths.native_out_dir,
                &cpp_library_name,
                &grammar.extra_c_flags,
                cpp_scanner_paths.clone(),
            );
        }

        write_fingerprint(&cache_paths.native_fingerprint_path, &native_fingerprint);
        profiler.log_stage(
            &grammar.name,
            "native",
            "rebuild",
            stage_started_at.elapsed(),
        );
    } else {
        profiler.log_stage(&grammar.name, "native", "cache-hit", Duration::ZERO);
    }

    profiler.log_stage(&grammar.name, "total", "done", grammar_started_at.elapsed());

    Some(GrammarBuildResult {
        native_out_dir: cache_paths.native_out_dir,
        library_name: grammar.library_name.clone(),
        cpp_library_name: (!cpp_scanner_paths.is_empty()).then_some(cpp_library_name),
    })
}

impl BuildProfiler {
    fn from_env() -> Self {
        let enabled = env::var("KAT_BUILD_PROFILE")
            .ok()
            .is_some_and(|value| value != "0" && !value.is_empty());
        let log_path = enabled.then(|| {
            env::var("KAT_BUILD_PROFILE_LOG")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    PathBuf::from(TREE_SITTER_CACHE_ROOT_DIR).join("tree-sitter-build-profile.log")
                })
        });

        Self { enabled, log_path }
    }

    fn log_global(&self, message: String) {
        if !self.enabled {
            return;
        }

        self.emit(format!("build-profile global {message}"));
    }

    fn log_stage(&self, grammar_name: &str, stage: &str, outcome: &str, elapsed: Duration) {
        if !self.enabled {
            return;
        }

        self.emit(format!(
            "build-profile grammar={} stage={} outcome={} elapsed_ms={}",
            grammar_name,
            stage,
            outcome,
            elapsed.as_millis()
        ));
    }

    fn emit(&self, line: String) {
        println!("cargo:warning={line}");

        let Some(log_path) = &self.log_path else {
            return;
        };

        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent)
                .unwrap_or_else(|error| panic!("failed to create {parent:?}: {error}"));
        }

        let mut content = line;
        content.push('\n');
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .unwrap_or_else(|error| panic!("failed to open {log_path:?}: {error}"));
        use std::io::Write;
        file.write_all(content.as_bytes())
            .unwrap_or_else(|error| panic!("failed to write {log_path:?}: {error}"));
    }
}

fn emit_grammar_source_watch_directives(grammar_dir: &Path) {
    for entry in WalkDir::new(grammar_dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();

        if !entry.file_type().is_file() || path.starts_with(grammar_dir.join("queries")) {
            continue;
        }

        if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                matches!(
                    extension,
                    "js" | "json" | "c" | "cc" | "cpp" | "h" | "hh" | "hpp"
                )
            })
        {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}

impl GrammarCachePaths {
    fn new(
        shared_cache_root: &Path,
        grammar_name: &str,
        build_target: &str,
        build_profile: &str,
    ) -> Self {
        let generated_root = shared_cache_root.join("generated").join(grammar_name);
        let staging_root = shared_cache_root
            .join("staging")
            .join(format!("pid-{}", process::id()))
            .join(grammar_name);
        let native_out_dir = shared_cache_root
            .join("native")
            .join(build_target)
            .join(build_profile);
        let grammar_native_out_dir = native_out_dir.join(grammar_name);

        Self {
            grammar_json_dir: generated_root.join("grammar-json"),
            grammar_json_staging_dir: staging_root.join("grammar-json"),
            grammar_json_fingerprint_path: generated_root.join("grammar-json.fingerprint"),
            parser_src_dir: generated_root.join("parser-src"),
            parser_src_staging_dir: staging_root.join("parser-src"),
            parser_fingerprint_path: generated_root.join("parser.fingerprint"),
            native_out_dir: grammar_native_out_dir.clone(),
            native_fingerprint_path: grammar_native_out_dir
                .join(format!("tree-sitter-{grammar_name}-native.fingerprint")),
        }
    }
}

fn grammar_job_count(grammar_count: usize) -> usize {
    let requested_jobs = env::var("KAT_BUILD_JOBS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0);

    let available_jobs = thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);

    requested_jobs
        .unwrap_or_else(|| available_jobs.min(8))
        .min(grammar_count.max(1))
}

fn regenerate_grammar_json(grammar_dir: &Path, cache_paths: &GrammarCachePaths) {
    remove_dir_if_exists(&cache_paths.grammar_json_staging_dir);
    fs::create_dir_all(&cache_paths.grammar_json_staging_dir).unwrap_or_else(|error| {
        panic!(
            "failed to create grammar staging dir {:?}: {error}",
            cache_paths.grammar_json_staging_dir
        )
    });

    generate_parser_in_directory(
        grammar_dir,
        Some(cache_paths.grammar_json_staging_dir.clone()),
        None::<PathBuf>,
        ABI_VERSION_MIN,
        None,
        None,
        false,
        OptLevel::default(),
    )
    .unwrap_or_else(|error| {
        panic!("failed to generate grammar.json from {grammar_dir:?}: {error}")
    });

    sync_generated_dir(
        &cache_paths.grammar_json_staging_dir,
        &cache_paths.grammar_json_dir,
    );
    remove_dir_if_exists(&cache_paths.grammar_json_staging_dir);
}

fn regenerate_parser_sources(
    grammar_dir: &Path,
    grammar_json_path: &Path,
    cache_paths: &GrammarCachePaths,
) {
    remove_dir_if_exists(&cache_paths.parser_src_staging_dir);
    fs::create_dir_all(&cache_paths.parser_src_staging_dir).unwrap_or_else(|error| {
        panic!(
            "failed to create parser staging dir {:?}: {error}",
            cache_paths.parser_src_staging_dir
        )
    });

    generate_parser_in_directory(
        grammar_dir,
        Some(cache_paths.parser_src_staging_dir.clone()),
        Some(grammar_json_path.to_path_buf()),
        ABI_VERSION_MIN,
        None,
        None,
        true,
        OptLevel::default(),
    )
    .unwrap_or_else(|error| {
        panic!("failed to generate parser.c from {grammar_json_path:?}: {error}")
    });

    sync_generated_dir(
        &cache_paths.parser_src_staging_dir,
        &cache_paths.parser_src_dir,
    );
    remove_dir_if_exists(&cache_paths.parser_src_staging_dir);
}

fn collect_scanner_paths(grammar_dir: &Path, names: &[&str]) -> Vec<PathBuf> {
    names
        .iter()
        .map(|name| grammar_dir.join(name))
        .filter(|path| path.exists())
        .inspect(|path| println!("cargo:rerun-if-changed={}", path.display()))
        .collect()
}

fn collect_support_paths(grammar_dir: &Path, extensions: &[&str]) -> Vec<PathBuf> {
    let mut paths = WalkDir::new(grammar_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .filter(|path| !path.starts_with(grammar_dir.join("queries")))
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extensions.iter().any(|expected| expected == &extension))
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn grammar_json_generation_fingerprint(grammar_input_paths: &[PathBuf]) -> Vec<u8> {
    let mut hasher = Hasher::new();
    hasher.update(b"kat-grammar-json-generation-v1");

    for path in grammar_input_paths {
        hash_file(&mut hasher, path);
    }

    hasher.finalize().as_bytes().to_vec()
}

fn parser_generation_fingerprint(grammar_json_path: &Path) -> Vec<u8> {
    let mut hasher = Hasher::new();
    hasher.update(b"kat-parser-generation-v1");
    hash_file(&mut hasher, grammar_json_path);
    hasher.update(ABI_VERSION_MIN.to_string().as_bytes());
    hasher.update(PARSER_OPT_LEVEL_NAME.as_bytes());
    hasher.finalize().as_bytes().to_vec()
}

fn native_compile_fingerprint(
    grammar: &GrammarSpec,
    parser_c_path: &Path,
    c_scanner_paths: &[PathBuf],
    cpp_scanner_paths: &[PathBuf],
    native_support_paths: &[PathBuf],
    grammar_dir: &Path,
    parser_src_dir: &Path,
) -> Vec<u8> {
    let mut hasher = Hasher::new();
    hasher.update(b"kat-native-compile-v1");
    hasher.update(grammar.name.as_bytes());
    hasher.update(&[0]);
    hasher.update(grammar.library_name.as_bytes());
    hasher.update(&[0]);
    hash_file(&mut hasher, parser_c_path);
    hash_file(&mut hasher, &parser_src_dir.join("tree_sitter/parser.h"));

    for scanner_path in c_scanner_paths {
        hash_file(&mut hasher, scanner_path);
    }

    for scanner_path in cpp_scanner_paths {
        hash_file(&mut hasher, scanner_path);
    }

    for support_path in native_support_paths {
        hash_file(&mut hasher, support_path);
    }

    for flag in &grammar.extra_c_flags {
        hasher.update(flag.as_bytes());
        hasher.update(&[0]);
    }

    hash_compiler_identity(
        &mut hasher,
        configured_compiler(false, &grammar.extra_c_flags, grammar_dir, parser_src_dir),
    );

    if !cpp_scanner_paths.is_empty() {
        hash_compiler_identity(
            &mut hasher,
            configured_compiler(true, &grammar.extra_c_flags, grammar_dir, parser_src_dir),
        );
    }

    hasher.finalize().as_bytes().to_vec()
}

fn configured_compiler(
    cpp: bool,
    extra_c_flags: &[String],
    grammar_dir: &Path,
    parser_src_dir: &Path,
) -> cc::Tool {
    let mut build = Build::new();
    build
        .cargo_metadata(false)
        .include(grammar_dir)
        .include(parser_src_dir);

    if cpp {
        build.cpp(true);
    } else {
        build.std("c11");
    }

    for flag in extra_c_flags {
        build.flag_if_supported(flag);
    }

    build.get_compiler()
}

fn hash_compiler_identity(hasher: &mut Hasher, tool: cc::Tool) {
    hasher.update(tool.path().as_os_str().as_encoded_bytes());
    hasher.update(&[0]);

    for arg in tool.args() {
        hasher.update(arg.as_encoded_bytes());
        hasher.update(&[0]);
    }

    for (key, value) in tool.env() {
        hasher.update(key.as_encoded_bytes());
        hasher.update(&[0]);
        hasher.update(value.as_encoded_bytes());
        hasher.update(&[0]);
    }
}

fn parser_artifacts_exist(parser_src_dir: &Path) -> bool {
    parser_src_dir.join("parser.c").exists() && parser_src_dir.join("tree_sitter/parser.h").exists()
}

fn fingerprint_matches(path: &Path, expected_fingerprint: &[u8]) -> bool {
    match fs::read(path) {
        Ok(actual_fingerprint) => actual_fingerprint == expected_fingerprint,
        Err(error) if error.kind() == ErrorKind::NotFound => false,
        Err(error) => panic!("failed to read {path:?}: {error}"),
    }
}

fn write_fingerprint(path: &Path, fingerprint: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("failed to create {parent:?}: {error}"));
    }

    fs::write(path, fingerprint)
        .unwrap_or_else(|error| panic!("failed to write {path:?}: {error}"));
}

fn remove_dir_if_exists(path: &Path) {
    match fs::remove_dir_all(path) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => panic!("failed to remove {path:?}: {error}"),
    }
}

fn hash_file(hasher: &mut Hasher, path: &Path) {
    hasher.update(path.as_os_str().as_encoded_bytes());
    hasher.update(&[0]);
    let bytes = fs::read(path).unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
    hasher.update(&bytes);
    hasher.update(&[0xff]);
}

fn library_archive_path(out_dir: &Path, library_name: &str) -> PathBuf {
    out_dir.join(format!("lib{library_name}.a"))
}

fn emit_link_directive(library_name: &str) {
    println!("cargo:rustc-link-lib=static={library_name}");
}

fn emit_cpp_runtime_link_directive() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("missing target os");

    let cpp_runtime = match target_os.as_str() {
        "macos" | "ios" | "tvos" | "watchos" => "c++",
        _ => "stdc++",
    };

    println!("cargo:rustc-link-lib={cpp_runtime}");
}

fn sync_generated_dir(source_dir: &Path, destination_dir: &Path) {
    fs::create_dir_all(destination_dir)
        .unwrap_or_else(|error| panic!("failed to create {destination_dir:?}: {error}"));

    let mut source_relative_paths = Vec::new();

    for entry in WalkDir::new(source_dir) {
        let entry = entry.unwrap_or_else(|error| panic!("failed to read generated entry: {error}"));
        let source_path = entry.path();
        let relative_path = source_path
            .strip_prefix(source_dir)
            .unwrap_or_else(|error| panic!("failed to strip prefix from {source_path:?}: {error}"));

        if relative_path.as_os_str().is_empty() {
            continue;
        }

        source_relative_paths.push(relative_path.to_path_buf());

        let destination_path = destination_dir.join(relative_path);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&destination_path)
                .unwrap_or_else(|error| panic!("failed to create {destination_path:?}: {error}"));
            continue;
        }

        copy_if_changed(source_path, &destination_path);
    }

    remove_stale_generated_files(destination_dir, &source_relative_paths);
}

fn copy_if_changed(source_path: &Path, destination_path: &Path) {
    if let Some(parent) = destination_path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("failed to create {parent:?}: {error}"));
    }

    let source_bytes = fs::read(source_path)
        .unwrap_or_else(|error| panic!("failed to read {source_path:?}: {error}"));

    let destination_matches = fs::read(destination_path)
        .map(|existing_bytes| existing_bytes == source_bytes)
        .unwrap_or(false);

    if destination_matches {
        return;
    }

    fs::write(destination_path, source_bytes)
        .unwrap_or_else(|error| panic!("failed to write {destination_path:?}: {error}"));
}

fn remove_stale_generated_files(destination_dir: &Path, source_relative_paths: &[PathBuf]) {
    for entry in WalkDir::new(destination_dir).contents_first(true) {
        let entry =
            entry.unwrap_or_else(|error| panic!("failed to read destination entry: {error}"));
        let destination_path = entry.path();
        let relative_path = destination_path
            .strip_prefix(destination_dir)
            .unwrap_or_else(|error| {
                panic!("failed to strip prefix from destination path {destination_path:?}: {error}")
            });

        if relative_path.as_os_str().is_empty() {
            continue;
        }

        if source_relative_paths
            .iter()
            .any(|source_relative_path| source_relative_path == relative_path)
        {
            continue;
        }

        if entry.file_type().is_dir() {
            fs::remove_dir(destination_path)
                .unwrap_or_else(|error| panic!("failed to remove {destination_path:?}: {error}"));
            continue;
        }

        fs::remove_file(destination_path)
            .unwrap_or_else(|error| panic!("failed to remove {destination_path:?}: {error}"));
    }
}

fn compile_c_sources(
    grammar_dir: &Path,
    generated_src_dir: &Path,
    out_dir: &Path,
    library_name: &str,
    extra_c_flags: &[String],
    source_paths: Vec<PathBuf>,
) {
    let mut build = Build::new();
    build.cargo_metadata(false);
    build
        .std("c11")
        .out_dir(out_dir)
        .include(grammar_dir)
        .include(generated_src_dir);

    for flag in extra_c_flags {
        build.flag_if_supported(flag);
    }

    for source_path in source_paths {
        build.file(source_path);
    }

    #[cfg(target_env = "msvc")]
    build.flag("-utf-8");

    build.compile(library_name);
}

fn compile_cpp_sources(
    grammar_dir: &Path,
    generated_src_dir: &Path,
    out_dir: &Path,
    library_name: &str,
    extra_c_flags: &[String],
    source_paths: Vec<PathBuf>,
) {
    let mut build = Build::new();
    build.cargo_metadata(false);
    build
        .cpp(true)
        .out_dir(out_dir)
        .include(grammar_dir)
        .include(generated_src_dir);

    for flag in extra_c_flags {
        build.flag_if_supported(flag);
    }

    for source_path in source_paths {
        build.file(source_path);
    }

    #[cfg(target_env = "msvc")]
    build.flag("-utf-8");

    build.compile(library_name);
}

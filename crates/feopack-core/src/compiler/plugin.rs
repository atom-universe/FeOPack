use super::Compiler;
use super::compilation::CompilationOptions;
use super::hooks::CompilerHooks;
use std::path::PathBuf;
use std::sync::Arc;

pub trait Plugin {
  fn apply(&self, context: &mut PluginApplyContext) -> Result<(), String>;
}

pub struct PluginApplyContext<'a> {
  pub(crate) compiler_hooks: &'a mut CompilerHooks,
  pub(crate) compiler_options: &'a CompilationOptions,
}

pub fn apply_builtin_plugin(name: &str, compiler: &mut Compiler) -> Result<(), String> {
  let mut context = PluginApplyContext {
    compiler_hooks: &mut compiler.hooks,
    compiler_options: &compiler.options,
  };

  match name {
    "traceLifecycle" | "trace-lifecycle" | "TraceLifecyclePlugin" => {
      TraceLifecyclePlugin.apply(&mut context)
    }
    "skipEmit" | "skip-emit" | "SkipEmitPlugin" => SkipEmitPlugin.apply(&mut context),
    _ => Err(format!("unknown feopack rust plugin: {}", name)),
  }
}

struct TraceLifecyclePlugin;

impl TraceLifecyclePlugin {
  fn append_line(log_file: &PathBuf, line: &str) -> Result<(), String> {
    if let Some(parent) = log_file.parent() {
      std::fs::create_dir_all(parent)
        .map_err(|e| format!("create hook log dir failed {:?}: {}", parent, e))?;
    }

    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
      .create(true)
      .append(true)
      .open(log_file)
      .map_err(|e| format!("open hook log failed {:?}: {}", log_file, e))?;
    writeln!(file, "{}", line).map_err(|e| format!("write hook log failed: {}", e))
  }
}

impl Plugin for TraceLifecyclePlugin {
  fn apply(&self, context: &mut PluginApplyContext) -> Result<(), String> {
    let log_file =
      Arc::new(PathBuf::from(&context.compiler_options.output.path).join("feopack-hooks.log"));
    match std::fs::remove_file(log_file.as_ref()) {
      Ok(()) => {}
      Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
      Err(err) => return Err(format!("remove old hook log failed: {}", err)),
    }

    let before_run_log = Arc::clone(&log_file);
    context
      .compiler_hooks
      .before_run
      .tap("TraceLifecyclePlugin", move |_| {
        Self::append_line(&before_run_log, "beforeRun")
      });

    let emit_log = Arc::clone(&log_file);
    context
      .compiler_hooks
      .emit
      .tap("TraceLifecyclePlugin", move |_| {
        Self::append_line(&emit_log, "emit")
      });

    let asset_emitted_log = Arc::clone(&log_file);
    context
      .compiler_hooks
      .asset_emitted
      .tap("TraceLifecyclePlugin", move |ctx| {
        Self::append_line(
          &asset_emitted_log,
          &format!("assetEmitted {}", ctx.filename),
        )
      });

    let done_log = Arc::clone(&log_file);
    context
      .compiler_hooks
      .done
      .tap("TraceLifecyclePlugin", move |_| {
        Self::append_line(&done_log, "done")
      });

    Ok(())
  }
}

struct SkipEmitPlugin;

impl Plugin for SkipEmitPlugin {
  fn apply(&self, context: &mut PluginApplyContext) -> Result<(), String> {
    context
      .compiler_hooks
      .should_emit
      .tap("SkipEmitPlugin", |_| Ok(Some(false)));
    Ok(())
  }
}

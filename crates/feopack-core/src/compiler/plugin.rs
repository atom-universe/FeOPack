use super::Compiler;
use super::compilation::CompilationOptions;
use super::hooks::CompilerHooks;
use std::path::PathBuf;
use std::sync::Arc;

pub trait Plugin: Send + Sync {
  fn apply(&self, context: &mut PluginApplyContext) -> Result<(), String>;
}

pub struct PluginApplyContext<'a> {
  pub(crate) compiler_hooks: &'a mut CompilerHooks,
  pub(crate) compiler_options: &'a CompilationOptions,
}

#[derive(Default)]
pub(crate) struct PluginDriver {
  compiler_hooks: CompilerHooks,
  #[allow(dead_code)]
  plugins: Vec<Box<dyn Plugin>>,
}

impl PluginDriver {
  pub(crate) fn compiler_hooks(&self) -> &CompilerHooks {
    &self.compiler_hooks
  }

  pub(crate) fn compiler_hooks_mut(&mut self) -> &mut CompilerHooks {
    &mut self.compiler_hooks
  }

  pub(crate) fn apply_builtin_plugin(
    &mut self,
    name: &str,
    compiler_options: &CompilationOptions,
  ) -> Result<(), String> {
    let plugin = create_builtin_plugin(name)?;
    self.apply_plugin(plugin, compiler_options)
  }

  fn apply_plugin(
    &mut self,
    plugin: Box<dyn Plugin>,
    compiler_options: &CompilationOptions,
  ) -> Result<(), String> {
    let mut context = PluginApplyContext {
      compiler_hooks: &mut self.compiler_hooks,
      compiler_options,
    };
    plugin.apply(&mut context)?;
    self.plugins.push(plugin);
    Ok(())
  }
}

fn create_builtin_plugin(name: &str) -> Result<Box<dyn Plugin>, String> {
  match name {
    "traceLifecycle" | "trace-lifecycle" | "TraceLifecyclePlugin" => {
      Ok(Box::new(TraceLifecyclePlugin))
    }
    "skipEmit" | "skip-emit" | "SkipEmitPlugin" => Ok(Box::new(SkipEmitPlugin)),
    _ => Err(format!("unknown feopack rust plugin: {}", name)),
  }
}

pub fn apply_builtin_plugin(name: &str, compiler: &mut Compiler) -> Result<(), String> {
  compiler
    .plugin_driver
    .apply_builtin_plugin(name, &compiler.options)
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

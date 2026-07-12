use super::Compiler;
use std::path::PathBuf;
use std::sync::Arc;

pub trait Plugin {
  fn apply(&self, compiler: &mut Compiler) -> Result<(), String>;
}

pub fn apply_builtin_plugin(name: &str, compiler: &mut Compiler) -> Result<(), String> {
  match name {
    "trace-lifecycle" => TraceLifecyclePlugin.apply(compiler),
    "skip-emit" => SkipEmitPlugin.apply(compiler),
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
  fn apply(&self, compiler: &mut Compiler) -> Result<(), String> {
    let log_file = Arc::new(PathBuf::from(&compiler.options.output.path).join("feopack-hooks.log"));
    match std::fs::remove_file(log_file.as_ref()) {
      Ok(()) => {}
      Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
      Err(err) => return Err(format!("remove old hook log failed: {}", err)),
    }

    let before_run_log = Arc::clone(&log_file);
    compiler
      .hooks_mut()
      .before_run
      .tap("TraceLifecyclePlugin", move |_| {
        Self::append_line(&before_run_log, "before_run")
      });

    let emit_log = Arc::clone(&log_file);
    compiler
      .hooks_mut()
      .emit
      .tap("TraceLifecyclePlugin", move |_| {
        Self::append_line(&emit_log, "emit")
      });

    let asset_emitted_log = Arc::clone(&log_file);
    compiler
      .hooks_mut()
      .asset_emitted
      .tap("TraceLifecyclePlugin", move |ctx| {
        Self::append_line(
          &asset_emitted_log,
          &format!("asset_emitted {}", ctx.filename),
        )
      });

    let done_log = Arc::clone(&log_file);
    compiler
      .hooks_mut()
      .done
      .tap("TraceLifecyclePlugin", move |_| {
        Self::append_line(&done_log, "done")
      });

    Ok(())
  }
}

struct SkipEmitPlugin;

impl Plugin for SkipEmitPlugin {
  fn apply(&self, compiler: &mut Compiler) -> Result<(), String> {
    compiler
      .hooks_mut()
      .should_emit
      .tap("SkipEmitPlugin", |_| Ok(Some(false)));
    Ok(())
  }
}

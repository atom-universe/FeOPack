use crate::loader::LoaderContext;

pub fn get_template(source: &str) -> Option<String> {
  let index1 = source.find("<meow>")?;
  let index2 = source.find("</meow>")?;
  let template = source[index1..index2+7].to_string();
  Some(template)
}

pub fn get_script(source: &str) -> Option<String> {
  let index1 = source.find("<script>")?;
  let index2 = source.find("</script>")?;
  let script = source[index1..index2+9].to_string();
  Some(script)
}

pub fn meow_loader_v1(context: LoaderContext) -> Result<String, String> {
  println!("meow_loader: {:?}", context.source);
  // TODO: 处理 context.source
  let template = get_template(&context.source).ok_or_else(|| "invalid .meow template format".to_string())?;
  let script = get_script(&context.source).ok_or_else(|| "invalid .meow script format".to_string())?;

  let handled_template = template
  .trim()
  .strip_prefix("<meow>")
  .and_then(|s| s.strip_suffix("</meow>"))
  .ok_or_else(|| "invalid .meow template format".to_string())?;

  let handled_script = script
  .trim()
  .strip_prefix("<script>")
  .and_then(|s| s.strip_suffix("</script>"))
  .ok_or_else(|| "invalid .meow script format".to_string())?;

  let handled_result = format!(
    r#"
const __feopack_meow_loader__ = () => {{ 
  const element = document.getElementById('meow');
  console.log(element);
  element.innerHTML={:?};

  // TODO: 后续改一下，只有带有 setup 的标签才会触发这个逻辑
  const script = document.createElement("script");
  script.textContent={:?};
  document.body.appendChild(script);
}};
export {{ __feopack_meow_loader__ as default }};"#,
  handled_template,
  handled_script
);

  Ok(handled_result)
}

// mini vue-loader
// .meow 文件 是简化的 .vue
/*
<meow>123</meow>

<script setup>
console.log(1)
</script>

<style scoped>
.meow {
  color: red;
}
</style>
*/

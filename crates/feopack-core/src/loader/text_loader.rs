pub fn text_loader(context: LoaderContext) -> Result<String, String> {
  Ok(format!("export default {:?}", context.source))
}

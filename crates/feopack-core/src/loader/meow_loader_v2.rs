use crate::loader::LoaderContext;

pub fn get_import_lines(source: &str) -> Vec<String> {
    let index1 = source.find("import")?;
}

pub fn meow_loader_v2(context: LoaderContext) -> Result<String, String> {
    const import_lines = get_import_lines(&context.source);


    Ok(format!(
        "const __feopack_meow_loader_v2__ = {:?};\nexport {{ __feopack_meow_loader_v2__ as default }};",
        context.source
    ))
}
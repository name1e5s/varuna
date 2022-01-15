use anyhow::{Context, Result};
use minijinja::{Environment, Source};
use rand::{thread_rng, Rng};
use std::{collections::BTreeMap, fs::read_dir, path::Path};

type Choice = BTreeMap<String, String>;

pub fn set_pre_compile_context(
    template_dir: &Path,
    context: &mut BTreeMap<String, String>,
) -> Result<Choice> {
    let mut env = Environment::new();
    let files = read_dir(template_dir).with_context(|| {
        format!(
            "failed to read template directory: {}",
            template_dir.display()
        )
    })?;
    let mut source = Source::new();
    let mut choice = BTreeMap::new();
    for file in files {
        let file = file?;
        let path = &file.path();
        if path.is_file() {
            set_file_source(path, None, &mut source, &mut choice)
                .with_context(|| format!("failed to set source for file: {}", path.display()))?;
        } else if path.is_dir() {
            set_dir_source(path, &mut source, &mut choice)?;
        } else {
            anyhow::bail!("unexpected file type: {}", path.display());
        }
    }
    env.set_source(source);
    for key in choice.keys() {
        let template = env
            .get_template(key)
            .with_context(|| format!("failed to get template: {}", key))?;
        let result = template
            .render(context.clone())
            .with_context(|| format!("failed to render template: {}", key))?;
        context.insert(key.to_string(), result);
    }
    Ok(choice)
}

fn set_file_source(
    path: &Path,
    name: Option<&String>,
    source: &mut Source,
    choice: &mut BTreeMap<String, String>,
) -> Result<()> {
    let file_name = path
        .file_prefix()
        .with_context(|| format!("failed to get file name: {}", path.display()))?
        .to_str()
        .with_context(|| format!("failed to convert file name to str: {}", path.display()))?
        .to_string();
    let name = name.cloned().unwrap_or(file_name);
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read file: {}", path.display()))?;
    choice.insert(name.clone(), path.to_string_lossy().to_string());
    source
        .add_template(name, content)
        .with_context(|| format!("failed to add template: {}", path.display()))?;
    Ok(())
}

fn set_dir_source(
    path: &Path,
    source: &mut Source,
    choice: &mut BTreeMap<String, String>,
) -> Result<()> {
    let name = path
        .file_name()
        .with_context(|| format!("failed to get dir name: {}", path.display()))?
        .to_str()
        .with_context(|| format!("failed to convert dir name to str: {}", path.display()))?
        .to_string();
    let paths = {
        let mut result = vec![];
        let files =
            read_dir(path).with_context(|| format!("failed to read dir: {}", path.display()))?;
        for file in files {
            let file = file?;
            let path = file.path();
            if path.is_file() {
                result.push(path);
            }
        }
        result
    };
    let rand_index: usize = thread_rng().gen_range(0..paths.len());
    let path = &paths[rand_index];
    set_file_source(path, Some(&name), source, choice)
        .with_context(|| format!("failed to set source for file: {}", path.display()))?;
    choice.insert(name, path.to_string_lossy().to_string());
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    #[test]
    fn test_pre_compile_context() {
        let mut context = BTreeMap::new();
        context.insert("id".to_string(), "114514".to_string());
        context.insert("serial".to_string(), "1919810".to_string());
        let choice =
            set_pre_compile_context(Path::new("example-package/template"), &mut context).unwrap();
        assert_eq!(context["id"], "114514");
        assert_eq!(context["serial"], "1919810");
        assert_eq!(context["hello"], "114514:1919810");
        if choice["test"].clone() == "example-package/template/test/testa.txt" {
            assert_eq!(context["test"], "testa:114514:1919810");
        } else {
            assert_eq!(context["test"], "testb:114514:1919810");
        }
    }
}

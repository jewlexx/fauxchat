use std::{path::Path, rc::Rc};

use deno_core::{error::AnyError, ModuleResolutionError, ModuleSpecifier};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    ModuleResolution(#[from] ModuleResolutionError),

    #[error("Please load the module using ChatScripts::load_module")]
    MissingModule,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct ChatScripts {
    current_module: Option<ModuleSpecifier>,
}

impl ChatScripts {
    pub fn new() -> Self {
        Self {
            current_module: None,
        }
    }

    pub fn load_module(&mut self, file_path: impl AsRef<Path>) -> Result<()> {
        let module = {
            let path = deno_core::normalize_path(file_path);
            deno_core::url::Url::from_file_path(&path)
                .map_err(|()| ModuleResolutionError::InvalidPath(path))
        }?;

        self.current_module = Some(module);

        Ok(())
    }

    pub async fn run_js(&self, file_path: &str) -> std::error::Result<(), AnyError> {
        let main_module = self.current_module.ok_or(Error::MissingModule)?;
        let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
            module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
            ..Default::default()
        });

        let mod_id = js_runtime.load_main_module(&main_module, None).await?;
        let result = js_runtime.mod_evaluate(mod_id);
        js_runtime.run_event_loop(false).await?;
        result.await?
    }
}

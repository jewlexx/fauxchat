#![warn(clippy::all, clippy::pedantic)]

use std::{
    path::Path,
    rc::Rc,
    sync::{Arc, OnceLock},
};

use deno_core::{error::AnyError, op, Extension, ModuleResolutionError, ModuleSpecifier};

static CALLBACK: OnceLock<Arc<dyn Fn() + Send + Sync>> = OnceLock::new();

#[op]
async fn op_send(
    message: String,
    count: usize,
    delay: u64,
    username: String,
) -> Result<(), AnyError> {
    todo!()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    ModuleResolution(#[from] ModuleResolutionError),

    #[error("Please load the module using ChatScripts::load_module")]
    MissingModule,

    #[error("Callback already set, maybe you have already created a ChatScripts struct?")]
    CallbackSet,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct ChatScripts {
    current_module: Option<ModuleSpecifier>,
}

impl ChatScripts {
    pub fn new(callback: impl Fn() + Send + Sync + 'static) -> Result<Self> {
        CALLBACK
            .set(Arc::new(callback))
            .map_err(|_| Error::CallbackSet)?;

        Ok(Self {
            current_module: None,
        })
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

    pub async fn run_js(&self) -> std::result::Result<(), AnyError> {
        let main_module = self.current_module.as_ref().ok_or(Error::MissingModule)?;

        let chat_extensions = Extension {
            name: "chat interactions",

            ..Default::default()
        };

        let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
            module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
            extensions: vec![chat_extensions],
            ..Default::default()
        });

        let mod_id = js_runtime.load_main_module(&main_module, None).await?;
        let result = js_runtime.mod_evaluate(mod_id);
        js_runtime.run_event_loop(false).await?;
        result.await?
    }
}

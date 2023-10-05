#![warn(clippy::all, clippy::pedantic)]

use std::{
    path::Path,
    rc::Rc,
    sync::{Arc, OnceLock},
};

use deno_core::{error::AnyError, FsModuleLoader, ModuleResolutionError, ModuleSpecifier};
use deno_runtime::deno_core;
use deno_runtime::{
    permissions::PermissionsContainer,
    worker::{MainWorker, WorkerOptions},
};

static CALLBACK: OnceLock<Arc<dyn Fn() + Send + Sync>> = OnceLock::new();

mod ops;

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
            ModuleSpecifier::from_file_path(&path)
                .map_err(|()| ModuleResolutionError::InvalidPath(path))
        }?;

        self.current_module = Some(module);

        Ok(())
    }

    pub async fn run_js(&self) -> std::result::Result<(), AnyError> {
        let module = self.current_module.as_ref().expect("loaded main module");
        let mut worker = MainWorker::bootstrap_from_options(
            module.clone(),
            PermissionsContainer::allow_all(),
            WorkerOptions {
                extensions: vec![ops::chat_commands::init_ops()],
                ..Default::default()
            },
        );

        worker.execute_script(
            "[chat_scripts:runtime.js]",
            deno_core::FastString::Static(include_str!("js/runtime.js")),
        )?;
        worker.execute_main_module(module).await?;
        worker.run_event_loop(false).await?;

        Ok(())
    }
}

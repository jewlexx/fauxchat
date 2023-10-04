#![warn(clippy::all, clippy::pedantic)]

use std::{
    path::Path,
    rc::Rc,
    sync::{Arc, OnceLock},
};

use deno_core::{error::AnyError, FsModuleLoader, ModuleSpecifier};
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
        let mut worker = MainWorker::bootstrap_from_options(
            self.current_module.clone(),
            PermissionsContainer::allow_all(),
            WorkerOptions {
                module_loader: Rc::new(FsModuleLoader),
                extensions: vec![chat_commands::init_ops()],
                ..Default::default()
            },
        );

        worker.execute_main_module(&main_module).await?;
        worker.run_event_loop(false).await?;
    }
}

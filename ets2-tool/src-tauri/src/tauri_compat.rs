//! Pure-Rust native runtime adapter replacing Tauri's `AppHandle`, `State`, `Manager`,
//! `Emitter`, `async_runtime`, and `tauri_plugin_dialog` with zero WebView dependencies.

use once_cell::sync::Lazy;
use serde::Serialize;
use serde_json::Value;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::future::Future;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock, mpsc};

pub trait Runtime: Send + Sync + Clone + 'static {}

#[derive(Debug, Clone, Copy, Default)]
pub struct NativeRuntime;

impl Runtime for NativeRuntime {}

#[derive(Debug, Clone)]
pub struct NativeEvent {
    pub event: String,
    pub payload: Value,
}

#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub identifier: String,
}

impl Default for PackageInfo {
    fn default() -> Self {
        Self {
            name: "Save-Edit Tool Native".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            identifier: "com.xlieferant.saveedittool".to_string(),
        }
    }
}

pub mod path {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BaseDirectory {
        Resource,
        Config,
        AppLocalData,
    }
}

#[derive(Debug, Clone, Default)]
pub struct PathResolver {
    identifier: String,
}

impl PathResolver {
    pub fn new(identifier: impl Into<String>) -> Self {
        Self {
            identifier: identifier.into(),
        }
    }

    pub fn resource_dir(&self) -> Result<PathBuf, String> {
        // Prefer the executable directory unconditionally (BD2): a
        // distributed binary may ship without sibling locales/ or
        // resources/, and resolving to the exe dir keeps lookups sane
        // (the translator falls back to raw keys when no locales are
        // found). The compile-time manifest dir stays only as a dev
        // fallback for runs where current_exe cannot be resolved.
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                return Ok(parent.to_path_buf());
            }
        }
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        if manifest_dir.exists() {
            return Ok(manifest_dir);
        }
        std::env::current_dir().map_err(|e| e.to_string())
    }

    pub fn config_dir(&self) -> Result<PathBuf, String> {
        dirs::config_dir()
            .ok_or_else(|| "System config directory could not be resolved.".to_string())
    }

    pub fn app_local_data_dir(&self) -> Result<PathBuf, String> {
        let base = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
        let id = if self.identifier.is_empty() {
            "com.xlieferant.saveedittool"
        } else {
            self.identifier.as_str()
        };
        Ok(base.join(id))
    }

    pub fn resolve(
        &self,
        subpath: impl AsRef<Path>,
        base: path::BaseDirectory,
    ) -> Result<PathBuf, String> {
        let root = match base {
            path::BaseDirectory::Resource => self.resource_dir()?,
            path::BaseDirectory::Config => self.config_dir()?,
            path::BaseDirectory::AppLocalData => self.app_local_data_dir()?,
        };
        Ok(root.join(subpath.as_ref()))
    }
}

pub struct State<'r, T: Send + Sync + 'static>(&'r T);

impl<'r, T: Send + Sync + 'static> State<'r, T> {
    pub fn new(reference: &'r T) -> Self {
        Self(reference)
    }

    pub fn inner(&self) -> &'r T {
        self.0
    }
}

impl<'r, T: Send + Sync + 'static> Clone for State<'r, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'r, T: Send + Sync + 'static> Copy for State<'r, T> {}

impl<'r, T: Send + Sync + 'static> Deref for State<'r, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

struct AppHandleInner {
    states: RwLock<HashMap<TypeId, &'static (dyn Any + Send + Sync)>>,
    listeners: Mutex<Vec<mpsc::Sender<NativeEvent>>>,
    path_resolver: PathResolver,
    package_info: PackageInfo,
}

pub struct AppHandle<R: Runtime = NativeRuntime> {
    inner: Arc<AppHandleInner>,
    _marker: std::marker::PhantomData<R>,
}

impl<R: Runtime> Clone for AppHandle<R> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            _marker: std::marker::PhantomData,
        }
    }
}

impl Default for AppHandle<NativeRuntime> {
    fn default() -> Self {
        Self::new()
    }
}

impl AppHandle<NativeRuntime> {
    pub fn new() -> Self {
        let package_info = PackageInfo::default();
        let path_resolver = PathResolver::new(package_info.identifier.clone());
        Self {
            inner: Arc::new(AppHandleInner {
                states: RwLock::new(HashMap::new()),
                listeners: Mutex::new(Vec::new()),
                path_resolver,
                package_info,
            }),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<R: Runtime> AppHandle<R> {
    pub fn subscribe_events(&self) -> mpsc::Receiver<NativeEvent> {
        let (tx, rx) = mpsc::channel();
        if let Ok(mut guard) = self.inner.listeners.lock() {
            guard.push(tx);
        }
        rx
    }

    pub fn handle(&self) -> &AppHandle<R> {
        self
    }

    pub fn path(&self) -> &PathResolver {
        &self.inner.path_resolver
    }

    pub fn package_info(&self) -> &PackageInfo {
        &self.inner.package_info
    }

    pub fn manage<T: Send + Sync + 'static>(&self, state: T) -> bool {
        let mut guard = self.inner.states.write().expect("state registry poisoned");
        let type_id = TypeId::of::<T>();
        if guard.contains_key(&type_id) {
            return false;
        }
        let leaked: &'static T = Box::leak(Box::new(state));
        guard.insert(type_id, leaked);
        true
    }

    pub fn try_state<T: Send + Sync + 'static>(&self) -> Option<State<'_, T>> {
        let guard = self.inner.states.read().ok()?;
        let any_ref = *guard.get(&TypeId::of::<T>())?;
        let typed_ref = any_ref.downcast_ref::<T>()?;
        Some(State::new(typed_ref))
    }

    pub fn state<T: Send + Sync + 'static>(&self) -> State<'_, T> {
        self.try_state::<T>().unwrap_or_else(|| {
            panic!(
                "Managed state `{}` not registered on AppHandle",
                std::any::type_name::<T>()
            )
        })
    }
}

pub trait Manager<R: Runtime = NativeRuntime> {
    fn handle(&self) -> &AppHandle<R>;
    fn path(&self) -> &PathResolver;
    fn package_info(&self) -> &PackageInfo;
    fn manage<T: Send + Sync + 'static>(&self, state: T) -> bool;
    fn try_state<T: Send + Sync + 'static>(&self) -> Option<State<'_, T>>;
    fn state<T: Send + Sync + 'static>(&self) -> State<'_, T>;
}

impl<R: Runtime> Manager<R> for AppHandle<R> {
    fn handle(&self) -> &AppHandle<R> {
        AppHandle::handle(self)
    }

    fn path(&self) -> &PathResolver {
        AppHandle::path(self)
    }

    fn package_info(&self) -> &PackageInfo {
        AppHandle::package_info(self)
    }

    fn manage<T: Send + Sync + 'static>(&self, state: T) -> bool {
        AppHandle::manage(self, state)
    }

    fn try_state<T: Send + Sync + 'static>(&self) -> Option<State<'_, T>> {
        AppHandle::try_state(self)
    }

    fn state<T: Send + Sync + 'static>(&self) -> State<'_, T> {
        AppHandle::state(self)
    }
}

pub trait Emitter<R: Runtime = NativeRuntime> {
    fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) -> Result<(), String>;
}

impl<R: Runtime> Emitter<R> for AppHandle<R> {
    fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) -> Result<(), String> {
        let value = serde_json::to_value(payload).map_err(|e| e.to_string())?;
        let native_event = NativeEvent {
            event: event.to_string(),
            payload: value,
        };
        if let Ok(mut listeners) = self.inner.listeners.lock() {
            listeners.retain(|sender| sender.send(native_event.clone()).is_ok());
        }
        Ok(())
    }
}

pub mod async_runtime {
    use super::*;

    static GLOBAL_RUNTIME: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to initialize global Tokio async runtime")
    });

    pub fn block_on<F: Future>(future: F) -> F::Output {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| handle.block_on(future))
        } else {
            GLOBAL_RUNTIME.block_on(future)
        }
    }

    pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        GLOBAL_RUNTIME.spawn(future)
    }

    pub fn spawn_blocking<F, R>(func: F) -> tokio::task::JoinHandle<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        GLOBAL_RUNTIME.spawn_blocking(func)
    }
}

pub mod dialog {
    use super::{AppHandle, NativeRuntime, Runtime};
    use std::path::{Path, PathBuf};

    #[derive(Debug, Clone)]
    pub struct FilePath(pub PathBuf);

    impl FilePath {
        pub fn into_path(self) -> Result<PathBuf, String> {
            Ok(self.0)
        }
    }

    #[derive(Default)]
    pub struct FileDialogBuilder {
        inner: rfd::FileDialog,
    }

    impl FileDialogBuilder {
        pub fn new() -> Self {
            Self {
                inner: rfd::FileDialog::new(),
            }
        }

        pub fn add_filter(mut self, name: impl Into<String>, extensions: &[&str]) -> Self {
            let name_str = name.into();
            self.inner = self.inner.add_filter(&name_str, extensions);
            self
        }

        pub fn set_title(mut self, title: impl Into<String>) -> Self {
            let title_str = title.into();
            self.inner = self.inner.set_title(&title_str);
            self
        }

        pub fn set_file_name(mut self, file_name: impl Into<String>) -> Self {
            let file_name_str = file_name.into();
            self.inner = self.inner.set_file_name(&file_name_str);
            self
        }

        pub fn set_directory(mut self, directory: impl AsRef<Path>) -> Self {
            self.inner = self.inner.set_directory(directory.as_ref());
            self
        }

        pub fn blocking_save_file(self) -> Option<FilePath> {
            self.inner.save_file().map(FilePath)
        }

        pub fn blocking_pick_file(self) -> Option<FilePath> {
            self.inner.pick_file().map(FilePath)
        }

        pub fn blocking_pick_folder(self) -> Option<FilePath> {
            self.inner.pick_folder().map(FilePath)
        }
    }

    #[derive(Default)]
    pub struct DialogBuilder;

    impl DialogBuilder {
        pub fn file(self) -> FileDialogBuilder {
            FileDialogBuilder::new()
        }
    }

    pub trait DialogExt<R: Runtime = NativeRuntime> {
        fn dialog(&self) -> DialogBuilder;
    }

    impl<R: Runtime> DialogExt<R> for AppHandle<R> {
        fn dialog(&self) -> DialogBuilder {
            DialogBuilder
        }
    }
}

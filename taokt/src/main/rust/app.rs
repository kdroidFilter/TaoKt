use std::{
    cell::Cell,
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use crate::{
    convert_event, ControlFlow, DeviceEventFilter, TaoError, TaoEvent, TaoUserEvent, Window, WindowBuilder,
};

thread_local! {
    static CURRENT_TARGET: Cell<*const tao::event_loop::EventLoopWindowTarget<TaoUserEvent>> = const { Cell::new(std::ptr::null()) };
}

struct TargetGuard;

impl TargetGuard {
    fn set(ptr: *const tao::event_loop::EventLoopWindowTarget<TaoUserEvent>) -> Self {
        CURRENT_TARGET.with(|cell| cell.set(ptr));
        Self
    }
}

impl Drop for TargetGuard {
    fn drop(&mut self) {
        CURRENT_TARGET.with(|cell| cell.set(std::ptr::null()));
    }
}

#[derive(uniffi::Record, Debug, Clone)]
pub struct RunConfig {
    pub device_event_filter: DeviceEventFilter,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            device_event_filter: DeviceEventFilter::Unfocused,
        }
    }
}

#[uniffi::export(callback_interface)]
pub trait TaoEventHandler {
    fn handle_event(&self, event: TaoEvent, app: Arc<App>) -> ControlFlow;
}

#[uniffi::export(callback_interface)]
pub trait TaoRunReturnHandler {
    fn handle_event(&self, event: TaoEvent, app: Arc<App>) -> ControlFlow;
    fn render(&self);
    fn should_quit(&self) -> bool;
}

#[derive(uniffi::Object)]
pub struct EventLoopProxy {
    inner: tao::event_loop::EventLoopProxy<TaoUserEvent>,
}

#[uniffi::export]
impl EventLoopProxy {
    pub fn send_event(&self, event: TaoUserEvent) -> Result<(), TaoError> {
        self.inner.send_event(event).map_err(|e| TaoError::message(format!("{e}")))?;
        Ok(())
    }
}

#[derive(uniffi::Object)]
pub struct App {
    proxy: tao::event_loop::EventLoopProxy<TaoUserEvent>,
    next_window_id: AtomicU64,
    window_ids: Mutex<HashMap<tao::window::WindowId, u64>>,
}

impl App {
    fn map_window_id(&self, id: tao::window::WindowId) -> u64 {
        let mut map = self.window_ids.lock().unwrap();
        if let Some(existing) = map.get(&id) {
            return *existing;
        }
        let next = self.next_window_id.fetch_add(1, Ordering::Relaxed);
        map.insert(id, next);
        next
    }

    fn with_target<R>(
        &self,
        f: impl FnOnce(&tao::event_loop::EventLoopWindowTarget<TaoUserEvent>) -> R,
    ) -> Result<R, TaoError> {
        CURRENT_TARGET.with(|cell| {
            let ptr = cell.get();
            if ptr.is_null() {
                return Err(TaoError::message(
                    "No active EventLoopWindowTarget (call this from within the event loop callback)",
                ));
            }
            // Safety: ptr is set for the duration of the event loop callback.
            Ok(f(unsafe { &*ptr }))
        })
    }
}

#[uniffi::export]
impl App {
    pub fn create_proxy(&self) -> Arc<EventLoopProxy> {
        Arc::new(EventLoopProxy {
            inner: self.proxy.clone(),
        })
    }

    pub fn create_window(&self, builder: Arc<WindowBuilder>) -> Result<Arc<Window>, TaoError> {
        self.with_target(|target| {
            let tao_builder = builder.clone_inner();
            tao_builder.build(target).map_err(TaoError::from)
        })?
        .map(|tao_window| {
            let tao_id = tao_window.id();
            let id = self.map_window_id(tao_id);
            Arc::new(Window {
                id,
                inner: Mutex::new(tao_window),
            })
        })
    }

    pub fn create_window_default(&self) -> Result<Arc<Window>, TaoError> {
        self.create_window(Arc::new(WindowBuilder::new()))
    }

    pub fn available_monitors(&self) -> Result<Vec<Arc<crate::Monitor>>, TaoError> {
        self.with_target(|target| {
            Ok(target
                .available_monitors()
                .map(|m| Arc::new(crate::Monitor { inner: m }))
                .collect())
        })?
    }

    pub fn primary_monitor(&self) -> Result<Option<Arc<crate::Monitor>>, TaoError> {
        self.with_target(|target| Ok(target.primary_monitor().map(|m| Arc::new(crate::Monitor { inner: m }))))?
    }
}

#[uniffi::export]
pub fn run(handler: Box<dyn TaoEventHandler>) {
    run_with_config(RunConfig::default(), handler)
}

#[uniffi::export]
pub fn run_with_config(config: RunConfig, handler: Box<dyn TaoEventHandler>) {
    let mut builder = tao::event_loop::EventLoopBuilder::<TaoUserEvent>::with_user_event();

    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    {
        use tao::platform::unix::EventLoopBuilderExtUnix;
        builder.with_any_thread(true);
    }

    #[cfg(target_os = "windows")]
    {
        use tao::platform::windows::EventLoopBuilderExtWindows;
        builder.with_any_thread(true);
    }

    let event_loop = builder.build();
    event_loop.set_device_event_filter(config.device_event_filter.into());

    let app = Arc::new(App {
        proxy: event_loop.create_proxy(),
        next_window_id: AtomicU64::new(1),
        window_ids: Mutex::new(HashMap::new()),
    });

    event_loop.run(move |event, target, control_flow| {
        let _guard = TargetGuard::set(target as *const _);
        let converted = convert_event(event, |id| app.map_window_id(id));
        if let Some(cf) = handler.handle_event(converted, app.clone()).to_tao() {
            *control_flow = cf;
        }
    });
}

#[uniffi::export]
pub fn run_return_loop(handler: Box<dyn TaoRunReturnHandler>) -> Result<(), TaoError> {
    run_return_loop_with_config(RunConfig::default(), handler)
}

#[uniffi::export]
pub fn run_return_loop_with_config(
    config: RunConfig,
    handler: Box<dyn TaoRunReturnHandler>,
) -> Result<(), TaoError> {
    #[cfg(target_os = "ios")]
    {
        let _ = config;
        let _ = handler;
        return Err(TaoError::Unsupported);
    }

    #[cfg(not(target_os = "ios"))]
    {
        use tao::platform::run_return::EventLoopExtRunReturn;

        let mut builder = tao::event_loop::EventLoopBuilder::<TaoUserEvent>::with_user_event();

        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            use tao::platform::unix::EventLoopBuilderExtUnix;
            builder.with_any_thread(true);
        }

        #[cfg(target_os = "windows")]
        {
            use tao::platform::windows::EventLoopBuilderExtWindows;
            builder.with_any_thread(true);
        }

        let mut event_loop = builder.build();
        event_loop.set_device_event_filter(config.device_event_filter.into());

        let app = Arc::new(App {
            proxy: event_loop.create_proxy(),
            next_window_id: AtomicU64::new(1),
            window_ids: Mutex::new(HashMap::new()),
        });

        while !handler.should_quit() {
            event_loop.run_return(|event, target, control_flow| {
                let _guard = TargetGuard::set(target as *const _);
                let converted = convert_event(event, |id| app.map_window_id(id));
                if let Some(cf) = handler.handle_event(converted, app.clone()).to_tao() {
                    *control_flow = cf;
                }
            });

            handler.render();
        }

        Ok(())
    }
}

use std::sync::{Arc, Mutex};

use crate::{PhysicalPositionI32, PhysicalSizeU32};

#[derive(uniffi::Object)]
pub struct Monitor {
    pub(crate) inner: tao::monitor::MonitorHandle,
}

#[uniffi::export]
impl Monitor {
    pub fn name(&self) -> Option<String> {
        self.inner.name()
    }

    pub fn size(&self) -> PhysicalSizeU32 {
        self.inner.size().into()
    }

    pub fn position(&self) -> PhysicalPositionI32 {
        self.inner.position().into()
    }

    pub fn scale_factor(&self) -> f64 {
        self.inner.scale_factor()
    }

    pub fn video_modes(&self) -> Vec<Arc<VideoMode>> {
        self.inner
            .video_modes()
            .map(|vm| Arc::new(VideoMode { inner: Mutex::new(vm) }))
            .collect()
    }

    pub fn debug_string(&self) -> String {
        format!("{:?}", self.inner)
    }
}

#[derive(uniffi::Object)]
pub struct VideoMode {
    pub(crate) inner: Mutex<tao::monitor::VideoMode>,
}

#[uniffi::export]
impl VideoMode {
    pub fn size(&self) -> PhysicalSizeU32 {
        let inner = self.inner.lock().unwrap();
        inner.size().into()
    }

    pub fn bit_depth(&self) -> u16 {
        let inner = self.inner.lock().unwrap();
        inner.bit_depth()
    }

    pub fn refresh_rate(&self) -> u16 {
        let inner = self.inner.lock().unwrap();
        inner.refresh_rate()
    }

    pub fn monitor(&self) -> Arc<Monitor> {
        let inner = self.inner.lock().unwrap();
        Arc::new(Monitor {
            inner: inner.monitor(),
        })
    }

    pub fn display_string(&self) -> String {
        let inner = self.inner.lock().unwrap();
        format!("{}", inner)
    }

    pub fn debug_string(&self) -> String {
        let inner = self.inner.lock().unwrap();
        format!("{:?}", inner)
    }
}

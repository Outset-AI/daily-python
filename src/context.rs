use std::ffi::CString;
use std::ptr;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::PyCustomAudioDevice;

use webrtc_daily::sys::{
    audio_device_module::NativeAudioDeviceModule, custom_audio_device::NativeCustomAudioDevice,
};

use daily_core::prelude::{
    daily_core_context_create_audio_device_module, daily_core_context_create_custom_audio_device,
    daily_core_context_custom_get_user_media, daily_core_context_select_custom_audio_device,
    WebrtcAudioDeviceModule, WebrtcPeerConnectionFactory, WebrtcTaskQueueFactory, WebrtcThread,
};

use pyo3::exceptions;
use pyo3::prelude::*;

// This should be initialized from Daily.init().
pub static mut GLOBAL_CONTEXT: Option<DailyContext> = None;

pub struct DailyContext {
    request_id: AtomicU64,
    audio_device_module: Option<NativeAudioDeviceModule>,
}

impl DailyContext {
    pub fn new() -> Self {
        Self {
            request_id: AtomicU64::new(0),
            audio_device_module: None,
        }
    }

    pub fn next_request_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn get_user_media(
        &mut self,
        peer_connection_factory: *mut WebrtcPeerConnectionFactory,
        signaling_thread: *mut WebrtcThread,
        worker_thread: *mut WebrtcThread,
        network_thread: *mut WebrtcThread,
        constraints: *const libc::c_char,
    ) -> *mut libc::c_void {
        if let Some(adm) = self.audio_device_module.as_mut() {
            daily_core_context_custom_get_user_media(
                adm.as_mut_ptr() as *mut _,
                peer_connection_factory,
                signaling_thread,
                worker_thread,
                network_thread,
                constraints,
            )
        } else {
            ptr::null_mut()
        }
    }

    pub fn create_audio_device_module(
        &mut self,
        task_queue_factory: *mut WebrtcTaskQueueFactory,
    ) -> *mut WebrtcAudioDeviceModule {
        unsafe {
            let adm = daily_core_context_create_audio_device_module(task_queue_factory);

            self.audio_device_module =
                Some(NativeAudioDeviceModule::from_unretained(adm as *mut _));

            adm
        }
    }

    pub fn create_custom_audio_device(
        &mut self,
        device_name: &str,
        play_sample_rate: u32,
        play_channels: u8,
        rec_sample_rate: u32,
        rec_channels: u8,
    ) -> PyResult<PyCustomAudioDevice> {
        if let Some(adm) = self.audio_device_module.as_mut() {
            let device_name_ptr = CString::new(device_name)
                .expect("invalid device name string")
                .into_raw();

            let mut device = PyCustomAudioDevice::new(
                device_name,
                play_sample_rate,
                play_channels,
                rec_sample_rate,
                rec_channels,
            );

            unsafe {
                let audio_device = daily_core_context_create_custom_audio_device(
                    adm.as_mut_ptr() as *mut _,
                    device_name_ptr,
                    play_sample_rate,
                    play_channels,
                    rec_sample_rate,
                    rec_channels,
                );

                device.attach_audio_device(NativeCustomAudioDevice::from_unretained(
                    audio_device as *mut _,
                ));

                let _ = CString::from_raw(device_name_ptr);
            }

            Ok(device)
        } else {
            Err(exceptions::PyRuntimeError::new_err(
                "custom audio module not created",
            ))
        }
    }

    pub fn select_custom_audio_device(&mut self, device_name: &str) -> PyResult<()> {
        if let Some(adm) = self.audio_device_module.as_ref() {
            let device_name_ptr = CString::new(device_name)
                .expect("invalid device name string")
                .into_raw();

            let selected = unsafe {
                let selected = daily_core_context_select_custom_audio_device(
                    adm.as_ptr() as *mut _,
                    device_name_ptr,
                );

                let _ = CString::from_raw(device_name_ptr);

                selected
            };

            if selected {
                Ok(())
            } else {
                Err(exceptions::PyRuntimeError::new_err(
                    "unable to select custom audio device",
                ))
            }
        } else {
            Err(exceptions::PyRuntimeError::new_err(
                "custom audio module not created",
            ))
        }
    }
}

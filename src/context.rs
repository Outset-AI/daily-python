use std::ffi::CString;
use std::ptr;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::PyCustomMicrophoneDevice;
use crate::PyCustomSpeakerDevice;

use webrtc_daily::sys::{
    audio_device_module::NativeAudioDeviceModule,
    custom_microphone_device::NativeCustomMicrophoneDevice,
    custom_speaker_device::NativeCustomSpeakerDevice, media_stream::MediaStream,
    rtc_refcount_interface_addref,
};

use daily_core::prelude::{
    daily_core_context_create_audio_device_module,
    daily_core_context_create_custom_microphone_device,
    daily_core_context_create_custom_speaker_device, daily_core_context_custom_get_user_media,
    daily_core_context_get_selected_custom_microphone_device,
    daily_core_context_select_custom_microphone_device,
    daily_core_context_select_custom_speaker_device, WebrtcAudioDeviceModule,
    WebrtcPeerConnectionFactory, WebrtcTaskQueueFactory, WebrtcThread,
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

    pub fn get_enumerated_devices(&self) -> *mut libc::c_char {
        if let Some(adm) = self.audio_device_module.as_ref() {
            let devices = unsafe {
                webrtc_daily::sys::webrtc_daily_custom_audio_enumerated_devices(
                    adm.as_ptr() as *mut _
                )
            };

            if devices.is_null() {
                concat!("[]", "\0").as_ptr() as *mut _
            } else {
                // TODO(aleix): leaking?
                devices as *mut _
            }
        } else {
            concat!("[]", "\0").as_ptr() as *mut _
        }
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
            if let Ok(mut media_stream) = MediaStream::new() {
                // Increase the reference count because it's decremented on drop
                // and we want to return a valid pointer.
                unsafe {
                    rtc_refcount_interface_addref(media_stream.as_mut_ptr());
                }

                media_stream.as_mut_ptr() as *mut _
            } else {
                ptr::null_mut()
            }
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

    pub fn create_speaker_device(
        &mut self,
        device_name: &str,
        sample_rate: u32,
        channels: u8,
    ) -> PyResult<PyCustomSpeakerDevice> {
        if let Some(adm) = self.audio_device_module.as_mut() {
            let device_name_ptr = CString::new(device_name)
                .expect("invalid speaker device name string")
                .into_raw();

            let mut py_device = PyCustomSpeakerDevice::new(device_name, sample_rate, channels);

            unsafe {
                let speaker_device = daily_core_context_create_custom_speaker_device(
                    adm.as_mut_ptr() as *mut _,
                    device_name_ptr,
                    sample_rate,
                    channels,
                );

                py_device.attach_audio_device(NativeCustomSpeakerDevice::from_unretained(
                    speaker_device as *mut _,
                ));

                let _ = CString::from_raw(device_name_ptr);
            }

            Ok(py_device)
        } else {
            Err(exceptions::PyRuntimeError::new_err(
                "custom audio module not created",
            ))
        }
    }

    pub fn create_microphone_device(
        &mut self,
        device_name: &str,
        sample_rate: u32,
        channels: u8,
    ) -> PyResult<PyCustomMicrophoneDevice> {
        if let Some(adm) = self.audio_device_module.as_mut() {
            let device_name_ptr = CString::new(device_name)
                .expect("invalid microphone device name string")
                .into_raw();

            let mut py_device = PyCustomMicrophoneDevice::new(device_name, sample_rate, channels);

            unsafe {
                let microphone_device = daily_core_context_create_custom_microphone_device(
                    adm.as_mut_ptr() as *mut _,
                    device_name_ptr,
                    sample_rate,
                    channels,
                );

                py_device.attach_audio_device(NativeCustomMicrophoneDevice::from_unretained(
                    microphone_device as *mut _,
                ));

                let _ = CString::from_raw(device_name_ptr);
            }

            Ok(py_device)
        } else {
            Err(exceptions::PyRuntimeError::new_err(
                "custom audio module not created",
            ))
        }
    }

    pub fn select_speaker_device(&mut self, device_name: &str) -> PyResult<()> {
        if let Some(adm) = self.audio_device_module.as_ref() {
            let device_name_ptr = CString::new(device_name)
                .expect("invalid speaker device name string")
                .into_raw();

            let selected = unsafe {
                let selected = daily_core_context_select_custom_speaker_device(
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
                    "unable to select custom speaker device",
                ))
            }
        } else {
            Err(exceptions::PyRuntimeError::new_err(
                "custom audio module not created",
            ))
        }
    }

    pub fn select_microphone_device(&mut self, device_name: &str) -> PyResult<()> {
        if let Some(adm) = self.audio_device_module.as_ref() {
            let device_name_ptr = CString::new(device_name)
                .expect("invalid microphone device name string")
                .into_raw();

            let selected = unsafe {
                let selected = daily_core_context_select_custom_microphone_device(
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
                    "unable to select custom microphone device",
                ))
            }
        } else {
            Err(exceptions::PyRuntimeError::new_err(
                "custom audio module not created",
            ))
        }
    }

    pub fn get_selected_microphone_device(&self) -> *const libc::c_char {
        if let Some(adm) = self.audio_device_module.as_ref() {
            let device =
                daily_core_context_get_selected_custom_microphone_device(adm.as_ptr() as *mut _);
            if device.is_null() {
                concat!("", "\0").as_ptr() as *const _
            } else {
                // TODO(aleix): leaking?
                device
            }
        } else {
            concat!("", "\0").as_ptr() as *const _
        }
    }
}

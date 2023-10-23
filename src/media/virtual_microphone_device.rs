use webrtc_daily::sys::virtual_microphone_device::NativeVirtualMicrophoneDevice;

use daily_core::prelude::daily_core_context_virtual_microphone_device_write_frames;

use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

/// This class represents a virtual microphone device. Virtual microphone
/// devices are used to send audio to the meeting.
///
/// The audio format used by virtual microphone devices is 16-bit linear PCM.
#[derive(Clone)]
#[pyclass(name = "VirtualMicrophoneDevice", module = "daily")]
pub struct PyVirtualMicrophoneDevice {
    device_name: String,
    sample_rate: u32,
    channels: u8,
    audio_device: Option<NativeVirtualMicrophoneDevice>,
}

impl PyVirtualMicrophoneDevice {
    pub fn new(device_name: &str, sample_rate: u32, channels: u8) -> Self {
        Self {
            device_name: device_name.to_string(),
            sample_rate,
            channels,
            audio_device: None,
        }
    }

    pub fn attach_audio_device(&mut self, audio_device: NativeVirtualMicrophoneDevice) {
        self.audio_device = Some(audio_device);
    }
}

#[pymethods]
impl PyVirtualMicrophoneDevice {
    /// Returns the device name.
    ///
    /// :return: The virtual microphone device name
    /// :rtype: str
    #[getter]
    fn name(&self) -> String {
        self.device_name.clone()
    }

    /// Returns the sample rate of this device (e.g. 16000).
    ///
    /// :return: The sample rate
    /// :rtype: int
    #[getter]
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Returns the number of channels (2 for stereo and 1 for mono) of this device.
    ///
    /// :return: The number of channels
    /// :rtype: int
    #[getter]
    fn channels(&self) -> u8 {
        self.channels
    }

    /// Writes audio frames to a virtual microphone device created with
    /// :func:`Daily.create_microphone_device`.
    ///
    /// The number of audio frames should be a multiple of 10ms worth of audio
    /// frames (considering the configured device sample rate). For example, if
    /// the sample rate is 16000 and there's only 1 channel, we should be able
    /// to write 160 audio frames (10ms), 320 (20ms), 480 (30ms), etc. If the
    /// number of audio frames is not a multiple of 10ms worth of audio frames,
    /// silence will be added as padding.
    ///
    /// To get low latency real time performance it is important that
    /// consecutive calls to this function don't take more time than the
    /// provided audio frames time. For example, if we provide audio frames
    /// every 10ms then we shouldn't take longer than 10ms to provide the next
    /// ones.
    ///
    /// :param bytestring frames: A bytestring with the audio frames to write
    ///
    /// :return: The number of audio frames written
    /// :rtype: int
    pub fn write_frames(&self, py: Python<'_>, frames: &PyBytes) -> PyResult<PyObject> {
        if let Some(audio_device) = self.audio_device.as_ref() {
            let bytes_length = frames.len()?;

            // libwebrtc needs 16-bit linear PCM samples
            if bytes_length % 2 != 0 {
                return Err(exceptions::PyValueError::new_err(
                    "frames bytestring should contain 16-bit samples",
                ));
            }

            let num_frames = bytes_length / 2; // 16 bits/sample / 8 bits/byte = 2 byte/sample

            // TODO(aleix): Should this be i16 aligned?
            let bytes = frames.as_bytes();

            let frames_written = py.allow_threads(move || unsafe {
                daily_core_context_virtual_microphone_device_write_frames(
                    audio_device.as_ptr() as *mut _,
                    bytes.as_ptr() as *const _,
                    num_frames,
                )
            });

            if frames_written > 0 {
                Ok(frames_written.into_py(py))
            } else {
                Err(exceptions::PyIOError::new_err(
                    "error writing audio frames to device",
                ))
            }
        } else {
            Err(exceptions::PyRuntimeError::new_err(
                "no microphone device has been attached",
            ))
        }
    }
}

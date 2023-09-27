use webrtc_daily::sys::virtual_speaker_device::NativeVirtualSpeakerDevice;

use daily_core::prelude::daily_core_context_virtual_speaker_device_read_frames;

use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

/// This class represents a virtual speaker device. Virtual speaker devices are
/// used to receive audio from the meeting.
///
/// The audio format used by virtual speaker devices is 16-bit linear PCM.
#[derive(Clone, Debug)]
#[pyclass(name = "VirtualSpeakerDevice", module = "daily")]
pub struct PyVirtualSpeakerDevice {
    device_name: String,
    sample_rate: u32,
    channels: u8,
    audio_device: Option<NativeVirtualSpeakerDevice>,
}

impl PyVirtualSpeakerDevice {
    pub fn new(device_name: &str, sample_rate: u32, channels: u8) -> Self {
        Self {
            device_name: device_name.to_string(),
            sample_rate,
            channels,
            audio_device: None,
        }
    }

    pub fn attach_audio_device(&mut self, audio_device: NativeVirtualSpeakerDevice) {
        self.audio_device = Some(audio_device);
    }
}

#[pymethods]
impl PyVirtualSpeakerDevice {
    /// Returns the device name.
    ///
    /// :return: The virtual speaker device name
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

    /// Reads audio frames from a virtual speaker device created with
    /// :func:`Daily.create_speaker_device`.
    ///
    /// The number of audio frames to read should be a multiple of 10ms worth of
    /// audio frames (considering the configured device sample rate). For
    /// example, if the sample rate is 16000 and there's only 1 channel, we
    /// should be able to read 160 audio frames (10ms), 320 (20ms), 480 (30ms),
    /// etc.
    ///
    /// :param int num_frames: The number of audio frames to read
    ///
    /// :return: The read audio frames as a bytestring. If no audio frames could be read, it returns an empty bytestring
    /// :rtype: bytestring.
    pub fn read_frames(&self, num_frames: usize) -> PyResult<PyObject> {
        if let Some(audio_device) = self.audio_device.as_ref() {
            // libwebrtc provides with 16-bit linear PCM
            let bits_per_sample = 16;
            let num_bytes = num_frames * (bits_per_sample * self.channels() as usize) / 8;
            let num_words = num_bytes / 2;

            let mut buffer: Vec<i16> = Vec::with_capacity(num_words);

            let frames_read = unsafe {
                daily_core_context_virtual_speaker_device_read_frames(
                    audio_device.as_ptr() as *mut _,
                    buffer.as_mut_ptr(),
                    num_frames,
                )
            };

            Python::with_gil(|py| {
                if frames_read == num_frames as i32 {
                    let py_bytes =
                        unsafe { PyBytes::from_ptr(py, buffer.as_ptr() as *const u8, num_bytes) };
                    Ok(py_bytes.into_py(py))
                } else if frames_read == 0 {
                    let empty_bytes: [u8; 0] = [];
                    let py_bytes = PyBytes::new(py, &empty_bytes);
                    Ok(py_bytes.into_py(py))
                } else {
                    Err(exceptions::PyIOError::new_err(
                        "error reading audio frames from the device",
                    ))
                }
            })
        } else {
            Err(exceptions::PyRuntimeError::new_err(
                "no speaker device has been attached",
            ))
        }
    }
}

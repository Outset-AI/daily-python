pub(crate) mod delegate;
pub(crate) mod event;
pub(crate) mod event_handler;

pub(crate) use event_handler::PyEventHandler;

use delegate::*;

use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    ptr,
    str::FromStr,
    sync::{Arc, Mutex},
};

use pyo3::{exceptions, prelude::*};
use serde_json::Value;

use webrtc_daily::sys::color_format::ColorFormat;

use daily_core::prelude::*;

use crate::{util::dict::DictValue, GLOBAL_CONTEXT};

#[derive(Clone)]
struct CallClientPtr {
    ptr: *mut CallClient,
}

impl CallClientPtr {
    fn as_ptr(&mut self) -> *mut CallClient {
        self.ptr
    }

    unsafe fn as_mut(&mut self) -> &mut CallClient {
        &mut *(self.ptr)
    }
}

unsafe impl Send for CallClientPtr {}

/// This class represents a call client. A call client is a participant of a
/// Daily meeting and it can receive audio and video from other participants in
/// the meeting as well as send audio and video. Multiple instances of call
/// clients can be created in the same application.
///
/// :param class event_handler: A subclass of :class:`daily.EventHandler`
#[pyclass(name = "CallClient", module = "daily")]
pub struct PyCallClient {
    call_client: CallClientPtr,
    inner: Arc<PyCallClientInner>,
    delegate_ctx_ptr: DelegateContextPtr,
}

#[pymethods]
impl PyCallClient {
    /// Create a new call client. The new call client can receive meeting events
    /// through an event handler.
    #[new]
    pub fn new(event_handler: Option<PyObject>) -> PyResult<Self> {
        let call_client = unsafe { daily_core_call_client_create() };
        if !call_client.is_null() {
            // Get initial values
            let active_speaker = unsafe { get_active_speaker(&mut (*call_client))? };
            let inputs = unsafe { get_inputs(&mut (*call_client))? };
            let participant_counts = unsafe { get_participant_counts(&mut (*call_client))? };
            let publishing = unsafe { get_publishing(&mut (*call_client))? };
            let subscriptions = unsafe { get_subscriptions(&mut (*call_client))? };
            let subscription_profiles = unsafe { get_subscription_profiles(&mut (*call_client))? };
            let network_stats = unsafe { get_network_stats(&mut (*call_client))? };

            let inner = Arc::new(PyCallClientInner {
                delegates: Mutex::new(PyCallClientDelegateFns {
                    on_event: Some(on_event),
                    on_video_frame: Some(on_video_frame),
                }),
                completions: Mutex::new(HashMap::new()),
                video_renderers: Mutex::new(HashMap::new()),
                // Non-blocking
                active_speaker: Mutex::new(active_speaker),
                inputs: Mutex::new(inputs),
                participant_counts: Mutex::new(participant_counts),
                publishing: Mutex::new(publishing),
                subscriptions: Mutex::new(subscriptions),
                subscription_profiles: Mutex::new(subscription_profiles),
                network_stats: Mutex::new(network_stats),
            });

            let delegate_ctx = Arc::new(DelegateContext {
                inner: inner.clone(),
                event_handler_callback: event_handler,
            });

            let delegate_ctx_ptr = Arc::into_raw(delegate_ctx);

            let client_delegate = NativeCallClientDelegate::new(
                NativeCallClientDelegatePtr::new(delegate_ctx_ptr as *mut libc::c_void),
                NativeCallClientDelegateFns::new(on_event_native, on_video_frame_native),
            );

            unsafe {
                daily_core_call_client_set_delegate(&mut (*call_client), client_delegate);
            }

            Ok(Self {
                inner,
                call_client: CallClientPtr { ptr: call_client },
                delegate_ctx_ptr: DelegateContextPtr {
                    ptr: delegate_ctx_ptr,
                },
            })
        } else {
            Err(exceptions::PyRuntimeError::new_err(
                "unable to create a CallClient() object",
            ))
        }
    }

    /// Join a meeting given the `meeting_url` and the optional `meeting_token`
    /// and `client_settings`. The client settings specifie inputs updates or
    /// publising settings.
    ///
    /// :param str meeting_url: The URL of the Daily meeting to join
    /// :param str meeting_token: Meeting token if needed. This is needed if the client is an owner of the meeting
    /// :param dict client_settings: See :ref:`ClientSettings`
    /// :param func completion: An optional completion callback with two parameters: (:ref:`CallClientJoinData`, :ref:`CallClientError`)
    #[pyo3(signature = (meeting_url, meeting_token = None, client_settings = None, completion = None))]
    pub fn join(
        &mut self,
        py: Python<'_>,
        meeting_url: &str,
        meeting_token: Option<&str>,
        client_settings: Option<PyObject>,
        completion: Option<PyObject>,
    ) {
        // Meeting URL
        let meeting_url_cstr = CString::new(meeting_url).expect("invalid meeting URL string");

        // Meeting token
        let meeting_token_cstr = meeting_token
            .map(|token| CString::new(token).expect("invalid meeting token string"))
            .or(None);

        // Client settings
        let client_settings_cstr = client_settings
            .map(|settings| {
                let settings_map: HashMap<String, DictValue> = settings.extract(py).unwrap();
                let settings_string = serde_json::to_string(&settings_map).unwrap();
                CString::new(settings_string).expect("invalid client settings string")
            })
            .or(None);

        unsafe {
            let request_id = self.maybe_register_completion(completion);
            daily_core_call_client_join(
                self.call_client.as_mut(),
                request_id,
                meeting_url_cstr.as_ptr(),
                meeting_token_cstr
                    .as_ref()
                    .map_or(ptr::null_mut(), |s| s.as_ptr()),
                client_settings_cstr
                    .as_ref()
                    .map_or(ptr::null_mut(), |s| s.as_ptr()),
            );
        }
    }

    /// Leave a previously joined meeting.
    ///
    /// :param func completion: An optional completion callback with two parameters: (None, :ref:`CallClientError`)
    #[pyo3(signature = (completion = None))]
    pub fn leave(&mut self, completion: Option<PyObject>) {
        let request_id = self.maybe_register_completion(completion);
        unsafe {
            daily_core_call_client_leave(self.call_client.as_mut(), request_id);
        }
    }

    /// Sets this client's user name. The user name is what other participants
    /// might be able to see as a description of this client.
    ///
    /// :param str user_name: This client's user name
    #[pyo3(signature = (user_name))]
    pub fn set_user_name(&mut self, user_name: &str) {
        let user_name_cstr = CString::new(user_name).expect("invalid user name string");

        let request_id = self.maybe_register_completion(None);
        unsafe {
            daily_core_call_client_set_user_name(
                self.call_client.as_mut(),
                request_id,
                user_name_cstr.as_ptr(),
            );
        }
    }

    /// Returns the current active speaker.
    ///
    /// :return: See :ref:`Participant`
    /// :rtype: dict
    pub fn active_speaker(&self) -> PyResult<PyObject> {
        Ok(self.inner.active_speaker.lock().unwrap().clone())
    }

    /// Returns the current participants in the meeting.
    ///
    /// :return: See :ref:`CallAllParticipants`
    /// :rtype: dict
    pub fn participants(&mut self, py: Python<'_>) -> PyResult<PyObject> {
        unsafe {
            let participants_ptr = daily_core_call_client_participants(self.call_client.as_mut());
            let participants_string = CStr::from_ptr(participants_ptr)
                .to_string_lossy()
                .into_owned();

            let participants: HashMap<String, DictValue> =
                serde_json::from_str(participants_string.as_str()).unwrap();

            Ok(participants.to_object(py))
        }
    }

    /// Returns the number of hidden and non-hidden participants in the meeting.
    ///
    /// :return: The number of participants in the meeting. See :ref:`ParticipantCounts`
    /// :rtype: dict
    pub fn participant_counts(&self) -> PyResult<PyObject> {
        Ok(self.inner.participant_counts.lock().unwrap().clone())
    }

    /// Updates remote participants.
    ///
    /// :param dict remote_participants: See :ref:`RemoteParticipantUpdates`
    /// :param func completion: An optional completion callback with two parameters: (None, :ref:`CallClientError`)
    #[pyo3(signature = (remote_participants, completion = None))]
    pub fn update_remote_participants(
        &mut self,
        py: Python<'_>,
        remote_participants: PyObject,
        completion: Option<PyObject>,
    ) {
        let remote_participants_map: HashMap<String, DictValue> =
            remote_participants.extract(py).unwrap();

        let remote_participants_string = serde_json::to_string(&remote_participants_map).unwrap();

        let remote_participants_cstr =
            CString::new(remote_participants_string).expect("invalid remote participants string");

        let request_id = self.maybe_register_completion(completion);

        unsafe {
            daily_core_call_client_update_remote_participants(
                self.call_client.as_mut(),
                request_id,
                remote_participants_cstr.as_ptr(),
            );
        }
    }

    /// Returns the current client inputs. The inputs define the call client
    /// video and audio sources (i.e. cameras and microphones).
    ///
    /// :return: See :ref:`InputSettings`
    /// :rtype: dict
    pub fn inputs(&self) -> PyResult<PyObject> {
        Ok(self.inner.inputs.lock().unwrap().clone())
    }

    /// Updates input settings. This function allows you to update the call
    /// client video and audio inputs.
    ///
    /// :param dict input_settings: See :ref:`InputSettings`
    /// :param func completion: An optional completion callback with two parameters: (:ref:`InputSettings`, :ref:`CallClientError`)
    #[pyo3(signature = (input_settings, completion = None))]
    pub fn update_inputs(
        &mut self,
        py: Python<'_>,
        input_settings: PyObject,
        completion: Option<PyObject>,
    ) {
        let input_settings_map: HashMap<String, DictValue> = input_settings.extract(py).unwrap();

        let input_settings_string = serde_json::to_string(&input_settings_map).unwrap();

        let input_settings_cstr =
            CString::new(input_settings_string).expect("invalid input settings string");

        let request_id = self.maybe_register_completion(completion);

        unsafe {
            daily_core_call_client_update_inputs(
                self.call_client.as_mut(),
                request_id,
                input_settings_cstr.as_ptr(),
            );
        }
    }

    /// Returns the current client publishing settings. The publishing settings
    /// specify if media should be published (i.e. sent) and, if so, how it
    /// should be sent (e.g. what resolutions or bitrate).
    ///
    /// :return: See :ref:`PublishingSettings`
    /// :rtype: dict
    pub fn publishing(&self) -> PyResult<PyObject> {
        Ok(self.inner.publishing.lock().unwrap().clone())
    }

    /// Updates publishing settings. This function allows you to update the call
    /// client video and audio publishing settings.
    ///
    /// :param dict publishing_settings: See :ref:`PublishingSettings`
    /// :param func completion: An optional completion callback with two parameters: (:ref:`PublishingSettings`, :ref:`CallClientError`)
    #[pyo3(signature = (publishing_settings, completion = None))]
    pub fn update_publishing(
        &mut self,
        py: Python<'_>,
        publishing_settings: PyObject,
        completion: Option<PyObject>,
    ) {
        let publishing_settings_map: HashMap<String, DictValue> =
            publishing_settings.extract(py).unwrap();

        let publishing_settings_string = serde_json::to_string(&publishing_settings_map).unwrap();

        let publishing_settings_cstr =
            CString::new(publishing_settings_string).expect("invalid publishing settings string");

        let request_id = self.maybe_register_completion(completion);

        unsafe {
            daily_core_call_client_update_publishing(
                self.call_client.as_mut(),
                request_id,
                publishing_settings_cstr.as_ptr(),
            );
        }
    }

    /// Returns the current client subscriptions. The client subscriptions is a
    /// dictionary containing specific subscriptions per remote participant.
    ///
    /// :return: See :ref:`ParticipantSubscriptions`
    /// :rtype: dict
    pub fn subscriptions(&self) -> PyResult<PyObject> {
        Ok(self.inner.subscriptions.lock().unwrap().clone())
    }

    /// Updates subscriptions and subscription profiles. This function allows
    /// you to update subscription profiles and at the same time assign specific
    /// subscription profiles to a participant and even change specific settings
    /// for some participants.
    ///
    /// :param dict participant_settings: See :ref:`ParticipantSubscriptions`
    /// :param dict profile_settings: See :ref:`SubscriptionProfileSettings`
    /// :param func completion: An optional completion callback with two parameters: (:ref:`ParticipantSubscriptions`, :ref:`CallClientError`)
    #[pyo3(signature = (participant_settings = None, profile_settings = None, completion = None))]
    pub fn update_subscriptions(
        &mut self,
        py: Python<'_>,
        participant_settings: Option<PyObject>,
        profile_settings: Option<PyObject>,
        completion: Option<PyObject>,
    ) {
        // Participant subscription settings
        let participant_settings_cstr = participant_settings
            .map(|settings| {
                let settings_map: HashMap<String, DictValue> = settings.extract(py).unwrap();
                let settings_string = serde_json::to_string(&settings_map).unwrap();
                CString::new(settings_string).expect("invalid participant settings string")
            })
            .or(None);

        // Profile settings
        let profile_settings_cstr = profile_settings
            .map(|settings| {
                let settings_map: HashMap<String, DictValue> = settings.extract(py).unwrap();
                let settings_string = serde_json::to_string(&settings_map).unwrap();
                CString::new(settings_string).expect("invalid profile settings string")
            })
            .or(None);

        let request_id = self.maybe_register_completion(completion);

        unsafe {
            daily_core_call_client_update_subscriptions(
                self.call_client.as_mut(),
                request_id,
                participant_settings_cstr
                    .as_ref()
                    .map_or(ptr::null(), |s| s.as_ptr()),
                profile_settings_cstr
                    .as_ref()
                    .map_or(ptr::null(), |s| s.as_ptr()),
            );
        }
    }

    /// Returns the current client subscription profiles. A subscription profile
    /// gives a set of subscription media settings a name.
    ///
    /// :return: See :ref:`SubscriptionProfileSettings`
    /// :rtype: dict
    pub fn subscription_profiles(&self) -> PyResult<PyObject> {
        Ok(self.inner.subscription_profiles.lock().unwrap().clone())
    }

    /// Updates subscription profiles.
    ///
    /// :param dict profile_settings: See :ref:`SubscriptionProfileSettings`
    /// :param func completion: An optional completion callback with two parameters: (:ref:`SubscriptionProfileSettings`, :ref:`CallClientError`)
    #[pyo3(signature = (profile_settings, completion = None))]
    pub fn update_subscription_profiles(
        &mut self,
        py: Python<'_>,
        profile_settings: PyObject,
        completion: Option<PyObject>,
    ) {
        let profile_settings_map: HashMap<String, DictValue> =
            profile_settings.extract(py).unwrap();

        let profile_settings_string = serde_json::to_string(&profile_settings_map).unwrap();
        let profile_settings_cstr =
            CString::new(profile_settings_string).expect("invalid profile settings string");

        let request_id = self.maybe_register_completion(completion);

        unsafe {
            daily_core_call_client_update_subscription_profiles(
                self.call_client.as_mut(),
                request_id,
                profile_settings_cstr.as_ptr(),
            );
        }
    }

    /// Updates the client permissions. This will only update permissions for
    /// this client and is only allowed if this client is the owner of the
    /// meeting.
    ///
    /// :param dict permissions: See :ref:`ParticipantPermissions`
    /// :param func completion: An optional completion callback with two parameters: (None, :ref:`CallClientError`)
    #[pyo3(signature = (permissions, completion = None))]
    pub fn update_permissions(
        &mut self,
        py: Python<'_>,
        permissions: PyObject,
        completion: Option<PyObject>,
    ) {
        let permissions_map: HashMap<String, DictValue> = permissions.extract(py).unwrap();

        let permissions_string = serde_json::to_string(&permissions_map).unwrap();
        let permissions_cstr =
            CString::new(permissions_string).expect("invalid permissions string");

        let request_id = self.maybe_register_completion(completion);

        unsafe {
            daily_core_call_client_update_permissions(
                self.call_client.as_mut(),
                request_id,
                permissions_cstr.as_ptr(),
            );
        }
    }

    /// Starts a recording, if recording is enabled for the current room.
    ///
    /// :param dict streaming_settings: See :ref:`StreamingSettings`
    /// :param str stream_id: A unique stream identifier. Multiple recording sessions can be started by specifying a unique ID
    /// :param str force_new: Whether to force a new recording
    /// :param func completion: An optional completion callback with two parameters: (None, :ref:`CallClientError`)
    #[pyo3(signature = (streaming_settings = None, stream_id = None, force_new = None, completion = None))]
    pub fn start_recording(
        &mut self,
        py: Python<'_>,
        streaming_settings: Option<PyObject>,
        stream_id: Option<&str>,
        force_new: Option<bool>,
        completion: Option<PyObject>,
    ) {
        let mut settings_map: HashMap<String, DictValue> = HashMap::new();

        if let Some(stream_id) = stream_id {
            settings_map.insert("instanceId".to_string(), DictValue(stream_id.into()));
        }
        if let Some(streaming_settings) = streaming_settings {
            let dict: HashMap<String, DictValue> = streaming_settings.extract(py).unwrap();
            let map = dict.iter().map(|(k, v)| (k.clone(), v.0.clone())).collect();
            settings_map.insert(
                "streamingSettings".to_string(),
                DictValue(Value::Object(map)),
            );
        }
        if let Some(force_new) = force_new {
            settings_map.insert("forceNew".to_string(), DictValue(force_new.into()));
        }

        let settings_cstr = if settings_map.is_empty() {
            None
        } else {
            let settings_string = serde_json::to_string(&settings_map).unwrap();
            Some(CString::new(settings_string).expect("invalid recording settings string"))
        };

        let request_id = self.maybe_register_completion(completion);

        unsafe {
            daily_core_call_client_start_recording(
                self.call_client.as_mut(),
                request_id,
                settings_cstr.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
            );
        }
    }

    /// Stops an ongoing recording. If multiple recording instances are running,
    /// each instance must be stopped individually by providing the unique
    /// stream ID.
    ///
    /// :param str stream_id: A unique stream identifier
    /// :param func completion: An optional completion callback with two parameters: (None, :ref:`CallClientError`)
    #[pyo3(signature = (stream_id = None, completion = None))]
    pub fn stop_recording(&mut self, stream_id: Option<&str>, completion: Option<PyObject>) {
        let stream_id_cstr = stream_id
            .map(|id| CString::new(id).expect("invalid stream id string"))
            .or(None);

        let request_id = self.maybe_register_completion(completion);

        unsafe {
            daily_core_call_client_stop_recording(
                self.call_client.as_mut(),
                request_id,
                stream_id_cstr
                    .as_ref()
                    .map_or(ptr::null_mut(), |s| s.as_ptr()),
            );
        }
    }

    /// Updates an ongoing recording. If multiple recording instances are
    /// running, each instance must be updated individually by providing the
    /// unique stream ID.
    ///
    /// :param dict update_settings: See :ref:`StreamingUpdateSettings`
    /// :param str stream_id: A unique stream identifier
    /// :param func completion: An optional completion callback with two parameters: (None, :ref:`CallClientError`)
    #[pyo3(signature = (update_settings, stream_id = None, completion = None))]
    pub fn update_recording(
        &mut self,
        py: Python<'_>,
        update_settings: PyObject,
        stream_id: Option<&str>,
        completion: Option<PyObject>,
    ) {
        let stream_id_cstr = stream_id
            .map(|id| CString::new(id).expect("invalid stream id string"))
            .or(None);

        let update_settings_map: HashMap<String, DictValue> = update_settings.extract(py).unwrap();
        let update_settings_string = serde_json::to_string(&update_settings_map).unwrap();
        let update_settings_cstr =
            CString::new(update_settings_string).expect("invalid recording settings string");

        let request_id = self.maybe_register_completion(completion);

        unsafe {
            daily_core_call_client_update_recording(
                self.call_client.as_mut(),
                request_id,
                update_settings_cstr.as_ptr(),
                stream_id_cstr
                    .as_ref()
                    .map_or(ptr::null_mut(), |s| s.as_ptr()),
            );
        }
    }

    /// Starts a transcription service. This can be done by meeting owners or
    /// transcription admins when transcription is enabled in the Daily domain.
    ///
    /// :param dict settings: See :ref:`TranscriptionSettings`
    /// :param func completion: An optional completion callback with two parameters: (None, :ref:`CallClientError`)
    #[pyo3(signature = (settings = None, completion = None))]
    pub fn start_transcription(
        &mut self,
        py: Python<'_>,
        settings: Option<PyObject>,
        completion: Option<PyObject>,
    ) {
        let settings_cstr = settings
            .map(|settings| {
                let settings_map: HashMap<String, DictValue> = settings.extract(py).unwrap();
                let settings_string = serde_json::to_string(&settings_map).unwrap();
                CString::new(settings_string).expect("invalid transcription settings string")
            })
            .or(None);

        let request_id = self.maybe_register_completion(completion);

        unsafe {
            daily_core_call_client_start_transcription(
                self.call_client.as_mut(),
                request_id,
                settings_cstr.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
            );
        }
    }

    /// Stops a currently running transcription service. This can be done by
    /// meeting owners or transcription admins when transcription is enabled in
    /// the Daily domain.
    ///
    /// :param func completion: An optional completion callback with two parameters: (None, :ref:`CallClientError`)
    #[pyo3(signature = (completion = None))]
    pub fn stop_transcription(&mut self, completion: Option<PyObject>) {
        let request_id = self.maybe_register_completion(completion);

        unsafe {
            daily_core_call_client_stop_transcription(self.call_client.as_mut(), request_id);
        }
    }

    /// Sends a message to other participants, or another specific participant,
    /// during the call.
    ///
    /// :param any message: The message to send (should be serializable to JSON)
    /// :param str participant: The participant to send the message to. Or `None` to broadcast the message
    /// :param func completion: An optional completion callback with two parameters: (None, :ref:`CallClientError`)
    #[pyo3(signature = (message, participant = None , completion = None))]
    pub fn send_app_message(
        &mut self,
        py: Python<'_>,
        message: PyObject,
        participant: Option<&str>,
        completion: Option<PyObject>,
    ) {
        let message_value: DictValue = message.extract(py).unwrap();
        let message_string = serde_json::to_string(&message_value.0).unwrap();
        let message_cstr = CString::new(message_string).expect("invalid message string");

        let participant_cstr = participant
            .map(|p| CString::new(p).expect("invalid participant string"))
            .or(None);

        let request_id = self.maybe_register_completion(completion);

        unsafe {
            daily_core_call_client_send_app_message(
                self.call_client.as_mut(),
                request_id,
                message_cstr.as_ptr(),
                participant_cstr
                    .as_ref()
                    .map_or(ptr::null(), |s| s.as_ptr()),
            );
        }
    }

    /// Returns the latest network statistics.
    ///
    /// :return: See :ref:`NetworkStats`
    /// :rtype: dict
    pub fn get_network_stats(&self) -> PyResult<PyObject> {
        Ok(self.inner.network_stats.lock().unwrap().clone())
    }

    /// Registers a video renderer for the given video source of the provided
    /// participant. The color format of the received frames can be chosen.
    ///
    /// :param str participant_id: The ID of the participant to receive video from
    /// :param function callback: A function or class method to be called on every received frame. It receives two arguments: the participant ID and a :class:`VideoFrame`
    /// :param str video_source: The video source of the remote participant to receive (e.g. `camera`, `screenVideo` or a custom track name)
    /// :param str color_format: The color format that frames should be received. See :ref:`ColorFormat`
    #[pyo3(signature = (participant_id, callback, video_source = "camera", color_format = "RGBA"))]
    pub fn set_video_renderer(
        &mut self,
        participant_id: &str,
        callback: PyObject,
        video_source: &str,
        color_format: &str,
    ) -> PyResult<()> {
        let participant_cstr = CString::new(participant_id).expect("invalid participant ID string");
        let video_source_cstr = CString::new(video_source).expect("invalid video source string");
        let color_format_cstr = CString::new(color_format).expect("invalid color format string");

        if ColorFormat::from_str(color_format).is_err() {
            return Err(exceptions::PyValueError::new_err(format!(
                "invalid color format '{color_format}'"
            )));
        }

        let request_id = self.maybe_register_completion(None);

        // Use the request_id as our renderer_id (it will be unique anyways) and
        // register the video renderer python callback.
        self.inner
            .video_renderers
            .lock()
            .unwrap()
            .insert(request_id, callback);

        unsafe {
            daily_core_call_client_set_participant_video_renderer(
                self.call_client.as_mut(),
                request_id,
                request_id,
                participant_cstr.as_ptr(),
                video_source_cstr.as_ptr(),
                color_format_cstr.as_ptr(),
            );
        }

        Ok(())
    }

    fn maybe_register_completion(&mut self, completion: Option<PyObject>) -> u64 {
        let request_id = unsafe { GLOBAL_CONTEXT.as_ref().unwrap().next_request_id() };

        if let Some(completion) = completion {
            self.inner
                .completions
                .lock()
                .unwrap()
                .insert(request_id, completion);
        }

        request_id
    }
}

impl Drop for PyCallClient {
    // GIL acquired
    fn drop(&mut self) {
        unsafe {
            // Cleanup delegates so they are not called during destroy.
            let mut delegates = self.inner.delegates.lock().unwrap();
            delegates.on_event.take();
            delegates.on_video_frame.take();

            let mut call_client = self.call_client.clone();

            // We know the GIL is acquired because it is acquired before
            // dropping a pyclass object.
            let py = Python::assume_gil_acquired();

            // Here we release the GIL so we can allow any event delegates to
            // finish. The event delegates will be waiting on the GIL and
            // execute at this point. But since we just cleanup the delegates
            // above, the events will actually be a no-op.
            py.allow_threads(move || {
                daily_core_call_client_destroy(call_client.as_ptr());
            });
        }

        // Cleanup the delegate context. The delegate context still has one
        // reference count (because of we drop it but increase it again every
        // time a delegate happens). After the client is destroyed it is safe to
        // simply get rid of it
        let _delegate_ctx = unsafe { Arc::from_raw(self.delegate_ctx_ptr.ptr) };
    }
}

unsafe fn get_active_speaker(call_client: &mut CallClient) -> PyResult<PyObject> {
    let active_speaker_ptr = daily_core_call_client_active_speaker(call_client);
    let active_speaker_string = CStr::from_ptr(active_speaker_ptr)
        .to_string_lossy()
        .into_owned();

    let active_speaker: Option<HashMap<String, DictValue>> =
        serde_json::from_str(active_speaker_string.as_str()).unwrap();

    Python::with_gil(|py| Ok(active_speaker.to_object(py)))
}

unsafe fn get_inputs(call_client: &mut CallClient) -> PyResult<PyObject> {
    let inputs_ptr = daily_core_call_client_inputs(call_client);
    let inputs_string = CStr::from_ptr(inputs_ptr).to_string_lossy().into_owned();

    let inputs: HashMap<String, DictValue> = serde_json::from_str(inputs_string.as_str()).unwrap();

    Python::with_gil(|py| Ok(inputs.to_object(py)))
}

unsafe fn get_participant_counts(call_client: &mut CallClient) -> PyResult<PyObject> {
    let participant_counts_ptr = daily_core_call_client_participant_counts(call_client);
    let participant_counts_string = CStr::from_ptr(participant_counts_ptr)
        .to_string_lossy()
        .into_owned();

    let participant_counts: HashMap<String, DictValue> =
        serde_json::from_str(participant_counts_string.as_str()).unwrap();

    Python::with_gil(|py| Ok(participant_counts.to_object(py)))
}

unsafe fn get_publishing(call_client: &mut CallClient) -> PyResult<PyObject> {
    let publishing_ptr = daily_core_call_client_publishing(call_client);
    let publishing_string = CStr::from_ptr(publishing_ptr)
        .to_string_lossy()
        .into_owned();

    let publishing: HashMap<String, DictValue> =
        serde_json::from_str(publishing_string.as_str()).unwrap();

    Python::with_gil(|py| Ok(publishing.to_object(py)))
}

unsafe fn get_subscriptions(call_client: &mut CallClient) -> PyResult<PyObject> {
    let subscriptions_ptr = daily_core_call_client_subscriptions(call_client);
    let subscriptions_string = CStr::from_ptr(subscriptions_ptr)
        .to_string_lossy()
        .into_owned();

    let subscriptions: HashMap<String, DictValue> =
        serde_json::from_str(subscriptions_string.as_str()).unwrap();

    Python::with_gil(|py| Ok(subscriptions.to_object(py)))
}

unsafe fn get_subscription_profiles(call_client: &mut CallClient) -> PyResult<PyObject> {
    let profiles_ptr = daily_core_call_client_subscription_profiles(call_client);
    let profiles_string = CStr::from_ptr(profiles_ptr).to_string_lossy().into_owned();

    let profiles: HashMap<String, DictValue> =
        serde_json::from_str(profiles_string.as_str()).unwrap();

    Python::with_gil(|py| Ok(profiles.to_object(py)))
}

unsafe fn get_network_stats(call_client: &mut CallClient) -> PyResult<PyObject> {
    let stats_ptr = daily_core_call_client_get_network_stats(call_client);
    let stats_string = CStr::from_ptr(stats_ptr).to_string_lossy().into_owned();

    let stats: HashMap<String, DictValue> = serde_json::from_str(stats_string.as_str()).unwrap();

    Python::with_gil(|py| Ok(stats.to_object(py)))
}

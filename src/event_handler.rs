#![allow(unused_variables)]

use pyo3::prelude::*;
use pyo3::types::PyTuple;

/// This a base class for event handlers. Event handlers are used to handle
/// events from the meeting, for example when a participant joins or leaves the
/// meeting or when the active speaker changes.
///
/// Event handlers are registered when creating a :class:`daily.CallClient` and
/// should be created as a subclass of this class. Since event handlers are
/// created as a subclass, there is no need implement all the handler methods.
#[derive(Clone, Debug)]
#[pyclass(name = "EventHandler", module = "daily", subclass)]
pub struct PyEventHandler;

#[pymethods]
impl PyEventHandler {
    // Since this is a base class it might be that subclasses have arguments in
    // their constructors. Those would be passed to new() even if we don't
    // really need them. So, in order to accept any subclass arguments we just
    // use a py_args extra positional arguments trick.
    #[new]
    #[pyo3(signature = (*args))]
    fn new(args: &PyTuple) -> PyResult<Self> {
        Ok(Self {})
    }

    /// Event emitted when the active speaker of the call has changed.
    ///
    /// :param dict participant: See :ref:`Participant`
    fn on_active_speaker_changed(&self, participant: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when a custom app message is received from another participant.
    ///
    /// :param string message: Message received from a remote participant
    /// :param string sender: Participant ID that sent the message
    fn on_app_message(&self, message: PyObject, sender: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when an audio device is plugged or removed.
    ///
    /// :param dict available_devices: See :ref:`AvailableDevices`
    fn on_available_devices_updated(&self, available_devices: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the call state changes, normally as a consequence of
    /// invocations to :func:`daily.CallClient.join` or
    /// :func:`daily.CallClient.leave`
    ///
    /// :param string state: See :ref:`CallState`
    fn on_call_state_updated(&self, state: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when an error occurs.
    ///
    /// :param string message: The error message
    fn on_error(&self, message: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the input settings are updated, normally as a
    /// consequence of invocations to :func:`daily.CallClient.join`,
    /// :func:`daily.CallClient.leave` or
    /// :func:`daily.CallClient.update_inputs`.
    ///
    /// :param dict inputs: See :ref:`InputSettings`
    fn on_inputs_updated(&self, input_settings: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted for all participants when a live stream encounters an
    /// error.
    ///
    /// :param string stream_id: The ID of the live stream that generated the error
    /// :param string message: The error message
    fn on_live_stream_error(&self, stream_id: PyObject, message: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted for all participants when a live stream starts.
    ///
    /// :param dict status: See :ref:`LiveStreamStatus`
    fn on_live_stream_started(&self, status: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted for all participants when a live stream stops.
    ///
    /// :param string stream_id: The ID of the live stream that was stopped
    fn on_live_stream_stopped(&self, stream_id: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted for all participants when a live stream encounters a
    /// warning.
    ///
    /// :param string stream_id: The ID of the live stream that generated the warning
    /// :param string message: The warning message
    fn on_live_stream_warning(&self, stream_id: PyObject, message: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the logging & telemetry backend updates the network
    /// statistics.
    ///
    /// :param dict stats: See :ref:`NetworkStats`
    fn on_network_stats_updated(&self, stats: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the participant count changes.
    ///
    /// :param dict stats: See :ref:`ParticipantCounts`
    fn on_participant_counts_updated(&self, counts: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when a participant joins the call.
    ///
    /// :param dict participant: See :ref:`Participant`
    fn on_participant_joined(&self, participant: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when a participant has left the call.
    ///
    /// :param dict participant: See :ref:`Participant`
    /// :param string reason: See :ref:`ParticipantLeftReason`
    fn on_participant_left(&self, participant: PyObject, reason: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when a participant is updated. This can mean either the
    /// participant's metadata was updated, or the tracks belonging to the
    /// participant changed.
    ///
    /// :param dict participant: See :ref:`Participant`
    fn on_participant_updated(&self, participant: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the publishing settings are updated, normally as a
    /// consequence of invocations to :func:`daily.CallClient.join`,
    /// :func:`daily.CallClient.update_publishing`.
    ///
    /// :param dict publishing_settings: See :ref:`PublishingSettings`
    fn on_publishing_updated(&self, publishing_settings: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when a recording error occurs.
    ///
    /// :param string stream_id: The ID of the recording that generated the error
    /// :param string message: The error message
    fn on_recording_error(&self, stream_id: PyObject, message: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted for all participants when a recording starts.
    ///
    /// :param dict status: See :ref:`RecordingStatus`
    fn on_recording_started(&self, status: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted for all participants when a recording stops.
    ///
    /// :param string stream_id: The ID of the live stream that was stopped
    fn on_recording_stopped(&self, stream_id: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the subscription profile settings are updated as a
    /// consequence of calls to
    /// :func:`daily.CallClient.update_subscription_profiles`.
    ///
    /// :param dict subscription_profiles: See :ref:`SubscriptionProfileSettings`
    fn on_subscription_profiles_updated(&self, subscription_profiles: PyObject) -> PyResult<()> {
        Ok(())
    }

    /// Event emitted when the subscription settings are updated as a
    /// consequence of calls to :func:`daily.CallClient.update_subscriptions`.
    ///
    /// :param dict subscriptions: See :ref:`ParticipantSubscriptions`
    fn on_subscriptions_updated(&self, subscriptions: PyObject) -> PyResult<()> {
        Ok(())
    }
}

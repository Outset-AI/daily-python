# Changelog

All notable changes to the **daily-python** SDK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added `punctuate` and `endpointing` fields to `TranscriptionSettings`.

- Added dialout support with `CallClient.start_dialout()` and
  `CallClient.stop_dialout()`.

### Changed

- Renamed `session_id` field to `participantId` in `TranscriptionMessage`.

### Deprecated

<!-- for soon-to-be removed functionality -->

- n/a

### Removed

- Removed `is_final`, `user_id` and `user_name` fields from
  `TranscriptionMessage`.

### Fixed

- Room deletion messages from the server are now properly handled.

- `CallClient.send_app_message(None)` now properly triggers a `ValueError`
  exception.

- If an invalid participant ID is passed to `CallClient.send_app_message()` it
  will now trigger a `ValueError` exception.

- Fixed an issue that would cause audio crackling and popping when using
  non-blocking devices.

- Fixed support for different audio sample rates and number of channels, other
  than 16000 and 1 channel.

- Don't quote the participant ID when passing the string to video/audio renderer
  callbacks.

- Fixed a potential crash on shutdown when using a virtual camera device.

- Emit `transcription-started` event if transcription is already started when
  joining the room.

### Performance

<!-- for performance-relevant changes -->

- n/a

### Security

<!-- for security-relevant changes -->

- n/a

### Other

- Added GStreamer media player demo.

## [0.5.4] - 2023-12-08

### Fixed

- Fixed another issue that could cause `CallClient.join()` to fail if another
  Daily web client was also joining at the same time.

## [0.5.3] - 2023-12-08

### Fixed

- Fixed an issue that could cause `CallClient.join()` to fail if another Daily
  web client was also joining at the same time.

## [0.5.2] - 2023-12-05

### Fixed

- Disabled echo cancellation, noise suppression and auto gain control by default
  to match the previous library behavior.

## [0.5.1] - 2023-11-30

### Fixed

- Fixed a crash when passing audio frames to `VirtualMicrophone.write_frames()`
  that require padding (i.e. non-multiple of 10ms worth of audio frames).

## [0.5.0] - 2023-11-30

### Added

- Support for non-blocking virtual audio devices. This allows integration with
  hardware devices (e.g. via PyAudio).

- Echo cancellation, noise suppression and auto gain control can now be enabled
  for virtual microphones via custom constraints.

- It is now possible to pass additional Deepgram settings to
  `start_transcription()` using the new `extra` field.

### Changed

- Transcription defaults have been removed in favor of Deepgram's defaults. This
  allows to simply specify `{"model": "nova-2"}`.

- Transcription `redact` can now also be a list of strings as supported by
  Deepgram (e.g. `["pci"]`).

### Fixed

- Fixed an issue on user leave (manual or by the server) that would prevent the
  user to rejoin.

### Other

- New demos to show how to integrate with PyAudio, how to send images and other
  improvements in existing demos.

## [0.4.0] - 2023-11-09

### Added

- Added support for capturing individual participant audio tracks.

- Added support for ejecting participants.

- Support python >= 3.7 and, on Linux, glibc >= 2.28.

### Changed

- Transcription defaults have been removed in favor of Deepgram's defaults. This
  allows to simply specify `{"model": "nova-2"}`.

- Transcription redact can now also be a list of strings as supported by
  Deepgram (e.g. `["pci"]`).

### Fixed

- Fixed a deadlock that would not allow receiving multiple simultaneous video
  renderers.

- Fixed a deadlock when a screen share was stopped.

- Fixed an issue where setting the user name could not be reflected
  automatically when requesting participants list.

- Fixed an issue that could cause joins/reconnects to not complete successfully.

- Fixed a sporadic crash that could occur when handling media streams.

### Performance

- Improved general video renderer performance.

- Improved media subscriptions stability and performance.

### Other

- Added Qt demo (similar to the existing Gtk demo).

- Qt and Gtk demos can now save the selected participant audio into a WAV file
  and can also render screen share.

## [0.3.1] - 2023-10-25

### Fixed

- Fixed an issue that could cause daily-python clients to join a session in a
  different region.

- Fixed a dead-lock that could occur when a `CallClient` is destroyed.

## [0.3.0] - 2023-10-23

### Added

- Support for sending chat messages to Daily Prebuilt
  (`CallClient.send_prebuilt_chat_message()`).

- Added Python type hints (helpful for editor completions).

- Support for Python 3.8.

### Changed

- `EventHandler.on_transcription_stopped` can now tell if transcription was
  stopped by a user or because of an error occurred.

### Removed

- Removed `detect_language` from `TranscriptionSettings`.

### Fixed

- Improved response time of `CallClient` getter functions.

- Improved low-latency performace of virtual audio devices.

- Fixed potential crash after `CallClient.leave()`.

- Improved internal safeness of participant video renderers.

- Fixed a `VirtualMicrophoneDevice` memory leak.

- Properly trigger a transcription error event if transcription can't start.

### Other

- Demos have been updated to show more real live code.

## [0.2.0] - 2023-10-03

### Added

- Support for start/stop recordings.

- Support for start/stop transcriptions and receive transcriptions messages.

### Changed

- `VirtualSpeakerDevice.read_frames()` has been improved and doesn't require the
  user to add sleeps. Therefore, it is now possible to read, for example, 10
  seconds of audio in a single call. Since the timings are now controlled
  internally, this minimizes any potential audio issues.

  The following old code:

```python
SAMPLE_RATE = 16000
READ_INTERVAL = 0.01
FRAMES_TO_READ = int(SAMPLE_RATE * READ_INTERVAL)
SECONDS_TO_READ = 10.0

for _ in range (int(SECONDS_TO_READ / READ_INTERVAL)):
  buffer = speaker.read_frames(FRAMES_TO_READ)
  time.sleep(READ_INTERVAL)
```

   can be replaced with:

```python
SECONDS_TO_READ = 10
FRAMES_TO_READ = SAMPLE_RATE * SECONDS_TO_READ
buffer = speaker.read_frames(FRAMES_TO_READ)
```

### Fixed

- Fixed an issue that was causing sporadic audio gaps on macOS and in certain OS
  task scheduling scenarios.

- Network re-connections have been improved.

## [0.1.1] - 2023-09-27

### Fixed

- Fixed an issue where virtual devices could cause other Python threads to be
  blocked.

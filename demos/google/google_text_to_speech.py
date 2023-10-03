#
# This demo requires Google Speech-To-Text credentials.
#
# See https://cloud.google.com/text-to-speech/docs/before-you-begin
#

from daily import *
from google.cloud import texttospeech

import argparse
import io
import time
import wave

parser = argparse.ArgumentParser()
parser.add_argument("-m", "--meeting", required = True, help = "Meeting URL")
parser.add_argument("-i", "--input", required = True, help = "File with sentences (one per line)")
args = parser.parse_args()

Daily.init()

# We create a virtual microphone device so we can read audio samples from the
# meeting.
microphone = Daily.create_microphone_device("my-mic", sample_rate = 16000, channels = 1)

client = CallClient()

# Here we tell our call client that we will be using our new virtual microphone.
client.update_inputs({
  "microphone": {
    "isEnabled": True,
    "settings": {
      "deviceId": "my-mic"
    }
  }
})

print()
print(f"Joining {args.meeting} ...")

client.join(args.meeting)

# Make sure we are joined. It would be better to use join() completion callback.
time.sleep(3)

sentences_file = open(args.input, "r")

voice = texttospeech.VoiceSelectionParams(
  language_code="en-US", name="en-US-Studio-M"
)

audio_config = texttospeech.AudioConfig(
  audio_encoding = texttospeech.AudioEncoding.LINEAR16,
  speaking_rate = 1.2,
  sample_rate_hertz = 16000
)

speech_client = texttospeech.TextToSpeechClient()

print()

for sentence in sentences_file.readlines():
  print(f"Processing: {sentence.strip()}")
  print()

  synthesis_input = texttospeech.SynthesisInput(text = sentence.strip())

  response = speech_client.synthesize_speech(
    input=synthesis_input, voice=voice, audio_config=audio_config
  )

  # Create an in-memory buffer with API's response.
  stream = io.BytesIO(response.audio_content)

  # The API response includes a WAV RIFF header, so we want to skip that since
  # that's not part of the audio samples.
  stream.read(44)

  # Send all the audio frames to the microphone.
  microphone.write_frames(stream.read())

# Let everything finish
time.sleep(2)

client.leave()

# Let leave finish
time.sleep(2)

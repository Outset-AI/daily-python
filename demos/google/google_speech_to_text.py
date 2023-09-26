#
# This demo requires Google Speech-To-Text credentials.
#
# See https://cloud.google.com/speech-to-text/docs/before-you-begin
#

from daily import *
from google.cloud import speech

import argparse
import io
import time
import wave

parser = argparse.ArgumentParser()
parser.add_argument("-m", "--meeting", required = True, help = "Meeting URL")
args = parser.parse_args()

Daily.init()

speaker = Daily.create_speaker_device("my-speaker", sample_rate = 16000, channels = 1)

Daily.select_speaker_device("my-speaker")

client = CallClient()

print()
print(f"Joining {args.meeting} ...")

client.join(args.meeting)

# Make sure we are joined. It would be better to use join() completion callback.
time.sleep(3)

SAMPLE_RATE = 16000
READ_INTERVAL = 1.0
SAMPLES_TO_READ = int(SAMPLE_RATE * READ_INTERVAL)
SECONDS_TO_READ = 10.0

print()
print(f"Now, say something in the meeting for {int(SECONDS_TO_READ)} seconds ...")

# We are creating a WAV file in memory so we can later grab the whole buffer and
# send it to Google Speech-To-Text API.
content = io.BufferedRandom(io.BytesIO())

out_wave = wave.open(content, "wb")
out_wave.setnchannels(1)
out_wave.setsampwidth(2) # 16-bit LINEAR PCM
out_wave.setframerate(16000)

# Here we are readin from the virtual speaker and writing the samples into the
# in-memory WAV file.
for _ in range (int(SECONDS_TO_READ / READ_INTERVAL)):
  buffer = speaker.read_samples(SAMPLES_TO_READ)
  out_wave.writeframesraw(buffer)
  time.sleep(READ_INTERVAL)

out_wave.close()

# We go to the beginning of the WAV buffer stream.
content.seek(0)

# We create and audio object with the contents of the in-memory WAV file.
audio = speech.RecognitionAudio(content = content.read())

# Configure Google Speech-To-Text so it receives 16-bit LINEAR PCM.
config = speech.RecognitionConfig(
  encoding=speech.RecognitionConfig.AudioEncoding.LINEAR16,
  sample_rate_hertz=16000,
  language_code="en-US",
)

speech_client = speech.SpeechClient()

print()
print(f"Transcribing with Google Speech-To-Text API ...")

response = speech_client.recognize(config=config, audio=audio)

print()
for result in response.results:
  print(f"Transcript: {result.alternatives[0].transcript}")

client.leave()

# Let leave finish
time.sleep(2)
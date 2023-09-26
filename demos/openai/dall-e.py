#
# This demo will join a Daily meeting, will listen for an audio prompt, will use
# Google Speech-To-Text API to translate that audio to text and then will use
# that text as a prompt for DALL-E to generate an image. The image will then be
# sent to the meeting using a virtual camera device.
#
# Usage: python dall-e.py -m MEETING_URL
#

from daily import *
from google.cloud import speech
from PIL import Image
import openai

import argparse
import io
import json
import os
import time
import wave
from base64 import b64decode

parser = argparse.ArgumentParser()
parser.add_argument("-m", "--meeting", required = True, help = "Meeting URL")
args = parser.parse_args()

openai.api_key = os.getenv("OPENAI_API_KEY")

Daily.init()

CAMERA_WIDTH = 1024
CAMERA_HEIGHT = 1024

speaker = Daily.create_speaker_device("my-speaker", sample_rate = 16000, channels = 1)
camera = Daily.create_camera_device("my-camera", width = CAMERA_WIDTH, height = CAMERA_HEIGHT, color_format = "RGB")

Daily.select_speaker_device("my-speaker")

client = CallClient()

client.update_inputs({
  "camera": {
    "isEnabled": True,
    "settings": {
      "deviceId": "my-camera"
    }
  },
  "microphone": False
})

print()
print(f"Joining {args.meeting} ...")

client.join(args.meeting)

# Make sure we are joined
time.sleep(3)

SAMPLE_RATE = 16000
READ_INTERVAL = 0.01
SAMPLES_TO_READ = int(SAMPLE_RATE * READ_INTERVAL)
SECONDS_TO_READ = 10.0

print()
print(f"Now, say something in the meeting for {int(SECONDS_TO_READ)} seconds ...")

content = io.BufferedRandom(io.BytesIO())

out_wave = wave.open(content, "wb")
out_wave.setnchannels(1)
out_wave.setsampwidth(2) # 16-bit LINEAR PCM
out_wave.setframerate(16000)

for _ in range (int(SECONDS_TO_READ / READ_INTERVAL)):
  buffer = speaker.read_samples(SAMPLES_TO_READ)
  out_wave.writeframesraw(buffer)
  time.sleep(READ_INTERVAL)

out_wave.close()

content.seek(0)

audio = speech.RecognitionAudio(content = content.read())

config = speech.RecognitionConfig(
  encoding=speech.RecognitionConfig.AudioEncoding.LINEAR16,
  sample_rate_hertz=16000,
  language_code="en-US",
)

speech_client = speech.SpeechClient()

print()
print(f"Transcribing with Google Speech-To-Text API ...")

response = speech_client.recognize(config=config, audio=audio)

if len(response.results) > 0 and len(response.results[0].alternatives) > 0:
  prompt = response.results[0].alternatives[0].transcript

  print()
  print(f"Generating image with OpenAI for '{prompt}' ...")

  response = openai.Image.create(
    prompt=prompt,
    n=1,
    size=f"{CAMERA_WIDTH}x{CAMERA_HEIGHT}",
    response_format="b64_json",
  )

  dalle_png = b64decode(response["data"][0]["b64_json"])

  dalle_stream = io.BytesIO(dalle_png)

  dalle_im = Image.open(dalle_stream)

  try:
    # This is a live video stream so we need to keep drawing the image.
    while True:
      camera.write_frame(dalle_im.tobytes())
      time.sleep(0.033)
  except KeyboardInterrupt:
    pass

client.leave()

# Let leave finish
time.sleep(2)

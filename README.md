# Tts Looper

A stupid little app to loop text to speech back into speech to text to see how badly the computer makes mistakes.

Uses DeepSpeech and flite as it's tts/stt engines

Environment setup currently requires a manual download of native_client...flite from https://github.com/mozilla/DeepSpeech/releases/tag/v0.9.0 to be added to LIBRARY_PATH and LD_LIBRARY_PATH

DeepSpeech model is tracked with git lfs

Requires Qt libs to be in the appropriate paths as well

![Example](demo/example.png)

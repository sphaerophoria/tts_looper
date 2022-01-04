#pragma once

#ifdef __cplusplus
extern "C" {
#endif
#include "stdbool.h"
#include "stdint.h"

typedef struct Gui Gui;

typedef struct String {
  const uint8_t* data;
  uint64_t len;
} String;

typedef struct GuiCallbacks {
  void (*start_tts_loop)(String text, int32_t num_iters, const void* data);
  void (*set_voice)(String voice, const void* data);
  void (*enable_audio)(bool enable, const void* data);
  void (*cancel)(const void* data);
  void (*start_recording)(const void* data);
  void (*end_recording)(const void* data);
  void (*save)(String path, const void* data);
} GuiCallbacks;

Gui* MakeGui(GuiCallbacks callbacks, const String* voices, uint64_t num_voices);
void DestroyGui(Gui* gui);

void PushOutput(Gui* gui, String text);
void PushRawOutput(Gui* gui, String text);
void PushInputText(Gui* gui, String text);

void Exec(Gui* gui, const void* data);

#ifdef __cplusplus
}
#endif

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
  void (*start_tts_loop)(String text, int32_t num_iters, bool play,
                         String voice, const void* data);
  void (*cancel)(const void* data);
} GuiCallbacks;

Gui* MakeGui(GuiCallbacks callbacks, const String* voices, uint64_t num_voices);
void DestroyGui(Gui* gui);

void PushLoopStart(Gui* gui, String text, String voice, int32_t num_iters);
void PushOutput(Gui* gui, String text);
void PushError(Gui* gui, String error);

void Exec(Gui* gui, const void* data);

#ifdef __cplusplus
}
#endif

#ifdef __cplusplus
extern "C" {
#endif
#include "stdbool.h"
#include "stdint.h"

typedef struct Gui Gui;

typedef struct GuiCallbacks {
    void(*start_tts_loop)(const uint8_t* text, uint64_t text_len, int32_t num_iters, bool play, const void* data);
} GuiCallbacks;

Gui* MakeGui(GuiCallbacks callbacks);
void DestroyGui(Gui* gui);

void ResetOutput(Gui* gui);
void PushOutput(Gui* gui, const uint8_t* text, uint64_t text_len);

void Exec(Gui* gui, const void* data);

#ifdef __cplusplus
}
#endif

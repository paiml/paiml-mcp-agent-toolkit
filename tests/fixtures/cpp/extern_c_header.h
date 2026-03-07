// Test fixture: Mixed extern "C" header (llama.h pattern)
#ifndef LLAMA_H
#define LLAMA_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

struct llama_context;
struct llama_model;

typedef struct llama_token_data {
    int32_t id;
    float logit;
    float p;
} llama_token_data;

int llama_model_load(const char* path, struct llama_model** model);
void llama_free(struct llama_context* ctx);
int llama_decode(struct llama_context* ctx, int n_tokens);

#ifdef __cplusplus
}
#endif

#ifdef __cplusplus
// C++ only API
namespace llama {
    class Context {
    public:
        Context(const char* model_path);
        ~Context();
        int decode(int n_tokens);
    private:
        struct llama_context* ctx_;
    };
}
#endif

#endif // LLAMA_H

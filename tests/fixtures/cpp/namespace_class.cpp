// Test fixture: Namespace and class qualified names
namespace llama {
namespace model {

class Transformer {
public:
    int load_weights(const char* path) {
        return 0;
    }

    void forward(float* input, float* output, int n) {
        for (int i = 0; i < n; i++) {
            output[i] = input[i];
        }
    }

private:
    int num_layers_;
};

int init_context(int n_ctx) {
    return n_ctx > 0 ? n_ctx : 512;
}

} // namespace model
} // namespace llama

namespace {
void anonymous_helper() {
    // Anonymous namespace function
}
} // anonymous namespace

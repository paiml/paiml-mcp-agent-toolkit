// Test fixture: Macro-heavy code (GGML-style)
#define GGML_ASSERT(x) if (!(x)) { abort(); }
#define GGML_TENSOR_LOCALS(type, prefix, t, i) \
    const int64_t prefix##0 = (t)->ne[0]; \
    const int64_t prefix##1 = (t)->ne[1];

void ggml_compute_forward_mul_mat(
    struct ggml_compute_params* params,
    struct ggml_tensor* dst) {

    struct ggml_tensor* src0 = dst->src[0];
    struct ggml_tensor* src1 = dst->src[1];

    GGML_ASSERT(params != NULL);
    GGML_ASSERT(src0 != NULL);
    GGML_ASSERT(src1 != NULL);
    GGML_ASSERT(dst != NULL);

    GGML_TENSOR_LOCALS(int64_t, ne0, src0, ne);
    GGML_TENSOR_LOCALS(int64_t, ne1, src1, ne);

    const int ith = params->ith;
    const int nth = params->nth;

    // Main computation loop
    for (int i = ith; i < ne01; i += nth) {
        for (int j = 0; j < ne11; j++) {
            float sum = 0.0f;
            for (int k = 0; k < ne00; k++) {
                sum += ((float*)src0->data)[i * ne00 + k] *
                       ((float*)src1->data)[j * ne10 + k];
            }
            ((float*)dst->data)[i * ne11 + j] = sum;
        }
    }
}

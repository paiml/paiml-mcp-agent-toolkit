// Test fixture: Template-heavy code (cutlass-style)
#include <type_traits>

template <int Arch, int Split, typename Element>
struct FlashAttention {
    static void run(const Element* Q, const Element* K, int seq_len) {
        // Template specialization
    }
};

template <typename T>
T clamp(T val, T min_val, T max_val) {
    return (val < min_val) ? min_val : (val > max_val) ? max_val : val;
}

template <int BlockM, int BlockN>
__global__ void tiled_matmul(float* C, const float* A, const float* B, int M, int N, int K) {
    __shared__ float As[BlockM][BlockN];
    __shared__ float Bs[BlockM][BlockN];

    int row = blockIdx.y * BlockM + threadIdx.y;
    int col = blockIdx.x * BlockN + threadIdx.x;

    float sum = 0.0f;
    for (int t = 0; t < K; t += BlockN) {
        As[threadIdx.y][threadIdx.x] = A[row * K + t + threadIdx.x];
        Bs[threadIdx.y][threadIdx.x] = B[(t + threadIdx.y) * N + col];
        __syncthreads();

        for (int k = 0; k < BlockN; k++) {
            sum += As[threadIdx.y][k] * Bs[k][threadIdx.x];
        }
        __syncthreads();
    }
    C[row * N + col] = sum;
}

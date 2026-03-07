// Test fixture: CUDA kernel with __global__/__device__/__shared__
#include <cuda_runtime.h>

__device__ float warp_reduce_sum(float val) {
    for (int offset = 16; offset > 0; offset /= 2) {
        val += __shfl_down_sync(0xffffffff, val, offset);
    }
    return val;
}

__global__ void softmax_kernel(float* output, const float* input, int n) {
    __shared__ float shared_max[32];
    __shared__ float shared_sum[32];

    int tid = threadIdx.x;
    int idx = blockIdx.x * blockDim.x + tid;

    // Find max
    float max_val = -INFINITY;
    if (idx < n) {
        max_val = input[idx];
    }
    shared_max[tid % 32] = max_val;
    __syncthreads();

    // Compute exp and sum
    float sum = 0.0f;
    if (idx < n) {
        sum = expf(input[idx] - max_val);
    }
    shared_sum[tid % 32] = sum;
    __syncthreads();

    // Normalize
    if (idx < n) {
        output[idx] = expf(input[idx] - max_val) / shared_sum[0];
    }
}

void launch_softmax(float* d_output, const float* d_input, int n) {
    int block_size = 256;
    int grid_size = (n + block_size - 1) / block_size;
    softmax_kernel<<<grid_size, block_size>>>(d_output, d_input, n);
}

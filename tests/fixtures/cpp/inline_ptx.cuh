// Test fixture: Inline PTX assembly (mma.cuh pattern from llama.cpp)
#pragma once

struct mma_int8 {
    static __device__ void mma_A(int* x, const int* A, const int* B) {
        asm("mma.sync.aligned.m16n8k32.row.col.s32.s8.s8.s32 "
            "{%0, %1, %2, %3}, "
            "{%4, %5, %6, %7}, "
            "{%8, %9}, "
            "{%10, %11, %12, %13};"
            : "=r"(x[0]), "=r"(x[1]), "=r"(x[2]), "=r"(x[3])
            : "r"(A[0]), "r"(A[1]), "r"(A[2]), "r"(A[3]),
              "r"(B[0]), "r"(B[1]),
              "r"(x[0]), "r"(x[1]), "r"(x[2]), "r"(x[3]));
    }
};

__device__ void async_copy(void* dst, const void* src) {
    asm volatile("cp.async.cg.shared.global [%0], [%1], 16;"
                 :: "r"((unsigned)__cvta_generic_to_shared(dst)),
                    "l"(src));
}

__device__ void barrier_sync() {
    asm volatile("bar.sync 0;");
}

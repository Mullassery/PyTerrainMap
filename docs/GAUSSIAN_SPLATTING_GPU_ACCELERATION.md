# GPU Acceleration Hints for Gaussian Splatting

## Overview

This document provides guidance for implementing GPU acceleration of Gaussian Splatting operations in PyTerrainMap. While the current CPU-based implementation achieves sub-millisecond latency for typical fleet sizes (100-10,000 splats), GPU acceleration enables:

- **Batch operations**: Process 1M+ observations per second
- **Large-scale fleets**: 50+ robots with 100k+ splats
- **Real-time uncertainty prediction**: Sub-10ms multi-region queries
- **Temporal decay**: Apply decay to entire store in <1ms

---

## Architecture: Compute Kernels by Operation

### Tier 1: Fusion Operations (Highest ROI)

**Current bottleneck:** Observation fusion (Bayesian position + covariance update) is the inner loop for multi-bot coordination.

```rust
// CPU version (3.2 microseconds per fusion)
pub fn fuse(existing: &mut TerrainGaussian, incoming: &TerrainGaussian) -> FusionResult {
    // 1. Position: weighted mean by inverse covariance
    let sigma_inv_1 = existing.covariance.inverse()?;
    let sigma_inv_2 = incoming.covariance.inverse()?;
    
    // new_pos = (Σ₁⁻¹·μ₁ + Σ₂⁻¹·μ₂) / (Σ₁⁻¹ + Σ₂⁻¹)
    let weighted_pos_1 = matrix_vec_mul(sigma_inv_1, existing.position);
    let weighted_pos_2 = matrix_vec_mul(sigma_inv_2, incoming.position);
    
    // 2. Covariance: (Σ₁⁻¹ + Σ₂⁻¹)⁻¹
    let combined_inv = matrix_add(sigma_inv_1, sigma_inv_2);
    let new_sigma = combined_inv.inverse()?;
    
    // 3. Traversability: weighted average by confidence
    // 4. Terrain type: majority vote
}
```

**GPU implementation (candidate):**

```cuda
// CUDA kernel: fuse_splats_batch
__global__ void fuse_splats_batch(
    const float3* existing_pos,           // [num_splats]
    const float3x3* existing_cov,         // [num_splats][3][3]
    const float* existing_conf,           // [num_splats]
    
    const float3* incoming_pos,           // [num_incoming]
    const float3x3* incoming_cov,         // [num_incoming][3][3]
    const float* incoming_conf,           // [num_incoming]
    
    int num_fusions,                      // matches in existing
    
    float3* out_pos,                      // [num_fusions]
    float3x3* out_cov,                    // [num_fusions][3][3]
    float* out_conf                       // [num_fusions]
) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= num_fusions) return;
    
    // Each thread fuses one splat pair
    float3x3 sigma_inv_1 = inverse_3x3(existing_cov[idx]);
    float3x3 sigma_inv_2 = inverse_3x3(incoming_cov[idx]);
    
    // Weighted position
    float3 pos1 = mat_vec_mul_3x3(sigma_inv_1, existing_pos[idx]);
    float3 pos2 = mat_vec_mul_3x3(sigma_inv_2, incoming_pos[idx]);
    float3 combined = add_float3(pos1, pos2);
    
    float3x3 sum_inv = add_mat_3x3(sigma_inv_1, sigma_inv_2);
    float3x3 new_cov = inverse_3x3(sum_inv);
    
    float3 new_pos = mat_vec_mul_3x3(new_cov, combined);
    
    // Confidence: agreement boost or reduction
    float conf_delta = (abs(existing_conf[idx] - incoming_conf[idx]) < 0.2) ? 0.05 : -0.03;
    float new_conf = fminf(1.0f, fmaxf(0.0f, (existing_conf[idx] + incoming_conf[idx]) / 2.0f + conf_delta));
    
    out_pos[idx] = new_pos;
    out_cov[idx] = new_cov;
    out_conf[idx] = new_conf;
}
```

**Speedup estimate:** 50-200x (1000s of fusions in parallel)

**Implementation steps:**
1. Batch observations by spatial proximity (use H3 grid)
2. Copy matching pairs to GPU memory
3. Launch `fuse_splats_batch` with grid=`(num_fusions/256, 1)`, block=`(256, 1)`
4. Copy results back
5. Overlap H2D/D2H transfers with kernel execution

---

### Tier 2: Spatial Queries (High ROI)

**Current bottleneck:** Radius queries scan all splats linearly O(n).

```rust
// CPU version: O(n) scan
pub fn query_radius(&self, center: [f64; 3], radius_m: f64) -> Vec<&TerrainGaussian> {
    self.splats.values()
        .filter(|splat| {
            let dist = haversine(center, splat.position);
            dist < radius_m
        })
        .collect()
}
```

**GPU implementation (candidate):**

```cuda
// CUDA kernel: query_radius_batch
__global__ void query_radius_batch(
    const float3* splat_pos,              // [num_splats]
    const float* splat_conf,              // [num_splats]
    float3 query_center,
    float radius_sq,                      // precomputed radius²
    int num_queries,
    const int3* query_indices,            // which splat per query
    
    uint32_t* output_mask,                // [num_splats] bitmask of matches
    int* output_count                     // number of matches
) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= num_splats) return;
    
    float3 delta = subtract_float3(splat_pos[idx], query_center);
    float dist_sq = dot_float3(delta, delta);
    
    if (dist_sq < radius_sq && splat_conf[idx] > 0.1f) {
        // Mark as match via atomic operation
        atomicOr(&output_mask[idx / 32], 1u << (idx % 32));
    }
    
    if (threadIdx.x == 0) {
        atomicAdd(output_count, __popc(output_mask[idx / 32]));
    }
}
```

**Speedup estimate:** 100-500x (10k splats in <1ms)

**Implementation steps:**
1. Copy all splat positions to constant memory (pinned transfer)
2. Iterate queries in batches (1000 at a time)
3. Use atomic operations for result compaction
4. Stream results back with concurrent CUDA stream

---

### Tier 3: Temporal Decay (Medium ROI)

**Current bottleneck:** Per-splat decay calculation (multiplicative per age).

```rust
// CPU version: O(n) decay
pub fn apply_decay(&mut self, current_time_us: i64, decay: &DecayFunction) {
    for splat in self.splats.values_mut() {
        let age_ms = (current_time_us - splat.last_updated) / 1000;
        let decayed = decay.apply(splat.confidence, age_ms);
        splat.confidence = decayed;
    }
}
```

**GPU implementation:**

```cuda
__global__ void apply_temporal_decay(
    float* confidences,                   // [num_splats] (in-place update)
    const int64_t* timestamps,            // [num_splats]
    int64_t current_time_us,
    float half_life_ms,                   // decay half-life
    int num_splats
) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= num_splats) return;
    
    int64_t age_ms = (current_time_us - timestamps[idx]) / 1000LL;
    float exponent = -0.693147f * (float)age_ms / half_life_ms;  // ln(0.5)
    float decayed = confidences[idx] * expf(exponent);
    
    confidences[idx] = decayed;
}
```

**Speedup estimate:** 50-100x (100k splats in ~1ms)

---

## Platform-Specific Implementations

### CUDA (NVIDIA GPUs)

**Target:** Data centers, high-performance workstations

- Use **Thrust library** for parallel operations (sort, scan, reduce)
- Use **cuBLAS** for batch matrix operations (inverse, multiply)
- Pinned memory for H2D/D2H transfers
- CUDA Streams for pipeline overlapping

**Example: Batch fusion with Thrust**

```cpp
#include <thrust/device_vector.h>
#include <thrust/sort.h>
#include <thrust/execution_policy.h>

void fuse_batch_thrust(
    thrust::device_vector<float3>& positions,
    thrust::device_vector<Matrix3f>& covariances,
    thrust::device_vector<float>& confidences,
    const std::vector<int>& match_indices  // indices of matching pairs
) {
    // Copy matches to device
    thrust::device_vector<int> d_matches(match_indices.begin(), match_indices.end());
    
    // Apply fuse kernel element-wise
    thrust::transform(
        thrust::cuda::par,
        d_matches.begin(), d_matches.end(),
        positions.begin(),
        fuse_transform_op{}  // custom functor
    );
}
```

**Compilation:**
```bash
nvcc -O3 -arch=sm_80 gaussian_splatting_kernels.cu -o libgs_cuda.so -Xcompiler -fPIC
```

---

### Metal (Apple Silicon)

**Target:** macOS + iPad M-series chips (PyTerrainMap's primary platform)

Use **Metal Performance Shaders (MPS)** and **Metal Compute**:

```swift
import MetalPerformanceShaders

class GaussianSplattingAccelerator {
    let device = MTLCreateSystemDefaultDevice()!
    let commandQueue = device.makeCommandQueue()!
    
    func fuseSplats(
        existingPositions: MTLBuffer,
        incomingPositions: MTLBuffer,
        matches: [Int]
    ) -> MTLBuffer {
        let commandBuffer = commandQueue.makeCommandBuffer()!
        
        // Use MPS matrix operations
        let mpsMatrixMultiplication = MPSMatrixMultiplication(
            device: device,
            transposeLeft: false,
            transposeRight: false,
            resultRows: 3,
            resultColumns: 1,
            interiorColumns: 3
        )
        
        // Batch execute
        for match_idx in matches {
            // Compute inverse covariance
            let descriptor = MPSMatrixCopyDescriptor(...)
            // Execute fusion kernel
        }
        
        commandBuffer.commit()
        commandBuffer.waitUntilCompleted()
        
        return resultBuffer
    }
}
```

**Speedup estimate:** 20-80x (M2 Pro / M3 Max)

---

### WebGPU (Browser-based Fleet Visualization)

**Target:** Real-time dashboard showing fleet state in browser

```javascript
// WebGPU compute shader for batch fusion
const fusionShader = `
@group(0) @binding(0) var<storage, read> existing_pos: array<vec3f>;
@group(0) @binding(1) var<storage, read> existing_cov: array<mat3x3f>;
@group(0) @binding(2) var<storage, read_write> out_pos: array<vec3f>;

@compute @workgroup_size(256)
fn fuse_kernel(@builtin(global_invocation_id) global_id: vec3u) {
    let idx = global_id.x;
    if (idx >= arrayLength(&out_pos)) { return; }
    
    // Compute inverse
    var sigma_inv: mat3x3f = inverse(existing_cov[idx]);
    
    // Weighted position (simplified)
    out_pos[idx] = existing_pos[idx];
}
`;

async function fuseBatch(existingPos, incomingPos) {
    const adapter = await navigator.gpu?.requestAdapter();
    const device = await adapter?.requestDevice();
    
    // Create buffers
    const posBuffer = device.createBuffer({
        size: existingPos.byteLength,
        usage: GPUBufferUsage.STORAGE,
        mappedAtCreation: true,
    });
    
    // Launch compute shader
    const computePass = commandEncoder.beginComputePass();
    computePass.setPipeline(pipeline);
    computePass.dispatchWorkgroups(Math.ceil(count / 256));
    computePass.end();
}
```

---

## Memory Layout Optimization

### Current CPU Layout (AoS: Array-of-Structs)
```rust
pub struct TerrainGaussian {
    pub position: [f64; 3],           // 24 bytes
    pub covariance: [[f32; 3]; 3],    // 36 bytes
    pub traversability: f32,          // 4 bytes
    pub terrain_type: TerrainType,    // 1-16 bytes (enum)
    pub confidence: f32,              // 4 bytes
    // ... more fields
}
```

**Problem:** Poor cache locality on GPU; warp divergence on `terrain_type`.

### Optimized GPU Layout (SoA: Struct-of-Arrays)
```rust
pub struct SplatBatch {
    positions: Vec<[f32; 3]>,         // N × 12 bytes (coalesced)
    covariances: Vec<[f32; 9]>,       // N × 36 bytes (coalesced)
    traversability: Vec<f32>,         // N × 4 bytes
    confidence: Vec<f32>,             // N × 4 bytes
    terrain_types: Vec<u8>,           // N × 1 byte
}
```

**Benefit:** 100% memory coalescing; 8-12x bandwidth utilization increase

---

## Parallelizable Operations Checklist

| Operation | Parallelism | GPU Kernel | Estimated 1M Splats |
|-----------|-------------|-----------|------|
| Fusion | Data-parallel (splat pairs) | Yes | ~1ms |
| Decay | Data-parallel (per-splat) | Yes | ~1ms |
| Radius query | Data-parallel (per-splat) | Yes | ~5ms |
| Uncertainty calc | Data-parallel (per-query point) | Yes | ~2ms |
| Terrain summary | Reduction (aggregation) | Yes | ~0.5ms |
| Covariance inverse | Data-parallel (3×3 per splat) | Yes | ~2ms |
| Mahalanobis distance | Data-parallel (pair-wise) | Yes | ~10ms |
| Temporal decay (weighted) | Data-parallel + atomic | Yes | ~3ms |
| LOD split/merge | Task-parallel (irregular) | Partial | ~50ms |
| Path cost (5-component) | Data-parallel (samples) | Yes | ~5ms |

---

## Integration Points

### 1. Fusion During Observation Ingestion
```rust
// src/gaussian_splatting/fleet_learning.rs
pub fn ingest_observation(&mut self, bot_id: &str, obs: Vec<ObjectObservation>) {
    #[cfg(feature = "gpu")]
    {
        // GPU: batch fusions
        let gpu_accel = GaussianGPUAccelerator::new();
        let fused = gpu_accel.fuse_batch(&self.store, &obs)?;
        self.store.apply_gpu_fusions(fused);
    }
    
    #[cfg(not(feature = "gpu"))]
    {
        // CPU fallback
        for observation in obs {
            self.store.insert_or_fuse(observation);
        }
    }
}
```

### 2. Bulk Decay During Store Maintenance
```rust
// src/gaussian_splatting/store.rs
pub fn apply_temporal_decay(&mut self, current_time_us: i64) {
    #[cfg(feature = "gpu")]
    {
        let gpu = GaussianGPUAccelerator::get();
        gpu.decay_all_splats(self, current_time_us)?;
    }
    
    #[cfg(not(feature = "gpu"))]
    {
        for splat in self.splats.values_mut() {
            splat.apply_decay(current_time_us);
        }
    }
}
```

### 3. Batch Uncertainty Queries for Path Planning
```rust
// src/gaussian_splatting/distance.rs
pub fn path_cost_batch(&self, paths: Vec<PathSegment>) -> Vec<PathCost> {
    #[cfg(feature = "gpu")]
    {
        let gpu = GaussianGPUAccelerator::get();
        gpu.compute_path_costs_batch(self.store, paths)?
    }
    
    #[cfg(not(feature = "gpu"))]
    {
        paths.into_iter().map(|p| self.path_cost(p)).collect()
    }
}
```

---

## Feature Flags

Add to `Cargo.toml`:

```toml
[features]
default = ["cpu"]
cpu = []
gpu-cuda = ["cudarc"]
gpu-metal = ["metal", "metal-rs"]
gpu-webgpu = ["wgpu"]
gpu = ["gpu-cuda"]  # default GPU backend

[dependencies]
cudarc = { version = "0.9", optional = true }
metal = { version = "0.27", optional = true }
wgpu = { version = "0.18", optional = true }
```

**Build variants:**
```bash
# CPU only (current)
cargo build

# CUDA enabled
cargo build --features gpu-cuda

# Metal enabled (macOS)
cargo build --features gpu-metal --target aarch64-apple-darwin

# WebGPU enabled
cargo build --features gpu-webgpu --target wasm32-unknown-unknown
```

---

## Profiling & Validation

### Benchmark Template

```rust
#[bench]
fn bench_fusion_cpu_vs_gpu(b: &mut Bencher) {
    let splats = gen_random_splats(10_000);
    let observations = gen_random_observations(10_000);
    
    // CPU baseline
    b.iter(|| {
        for obs in &observations {
            store.insert_or_fuse(obs.clone());
        }
    });
}

#[cfg(feature = "gpu-cuda")]
#[bench]
fn bench_fusion_gpu(b: &mut Bencher) {
    let gpu = GaussianGPUAccelerator::new();
    let splats = gen_random_splats(10_000);
    let observations = gen_random_observations(10_000);
    
    b.iter(|| {
        gpu.fuse_batch_async(&splats, &observations).wait();
    });
}
```

### Correctness Validation

```rust
#[test]
fn test_gpu_fusion_matches_cpu() {
    let splat_cpu = TerrainGaussian { ... };
    let obs_cpu = TerrainGaussian { ... };
    
    // CPU result
    let mut result_cpu = splat_cpu.clone();
    ObservationFuser::fuse(&mut result_cpu, &obs_cpu);
    
    // GPU result
    #[cfg(feature = "gpu-cuda")]
    {
        let gpu = GaussianGPUAccelerator::new();
        let result_gpu = gpu.fuse_single(splat_cpu, obs_cpu)?;
        
        // Compare (with tolerance for float error)
        assert!((result_cpu.position[0] - result_gpu.position[0]).abs() < 1e-5);
        assert!((result_cpu.confidence - result_gpu.confidence).abs() < 1e-4);
    }
}
```

---

## Phase-In Strategy

### Phase 1 (Week 1-2): Fusion Acceleration
- Implement `fuse_batch` kernel (CUDA + Metal)
- Add feature flag `gpu-cuda`
- Measure 50-100x speedup on 10k splats
- Benchmark real warehouse scenario (Task 12)

### Phase 2 (Week 3): Query Acceleration
- Implement radius query kernel
- Add spatial index precomputation on GPU
- Measure query latency improvement

### Phase 3 (Week 4): Decay & Prediction
- Implement temporal decay kernel
- Batch unknown region prediction
- End-to-end integration testing

### Phase 4 (Week 5+): WebGPU & Mobile
- WebGPU compute shaders for browser dashboard
- Metal optimization for iPad deployment
- Production profiling & tuning

---

## References

- **CUDA Programming Guide**: https://docs.nvidia.com/cuda/cuda-c-programming-guide/
- **Metal Performance Shaders**: https://developer.apple.com/metal/
- **WebGPU Spec**: https://www.w3.org/TR/webgpu/
- **Thrust Library**: https://github.com/NVIDIA/thrust
- **cuBLAS**: https://docs.nvidia.com/cuda/cublas/

---

*GPU acceleration is optional but recommended for fleets with >1k splats or sub-5ms latency requirements.*

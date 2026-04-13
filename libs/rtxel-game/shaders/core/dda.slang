// https://m4xc.dev/articles/amanatides-and-woo/
// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
// pretty much stolen from here, just reworked a bit to allow easier usage with
// multiple volumes

import ray;
import aabb;
import volume;

public struct DDAHitInfo {
    public HitInfo hitInfo;
    public AABB voxelAABB;
    public int3 gridPos;
    public int axis;

    public float3 normal(Ray ray) {
        var normal = float3(0.0);
        normal[axis] = sign(ray.direction[axis]);
        return normal;
    }
}

#define STEP_ALONG_AXIS(axis_, axis_index)                                     \
    do {                                                                       \
        state.pos.axis_ += int(state.step.axis_);                              \
        if (state.pos.axis_ < 0 || state.pos.axis_ >= state.grid.size)         \
            return false;                                                      \
        state.axis = axis_index;                                               \
        state.tmax.axis_ += state.delta.axis_;                                 \
    } while (false)

public struct DDAState {
    public float t;
    public float entry_t;
    public int3 pos;
    public float3 tmax;
    public int axis;
    public bool valid;

    public float3 step;
    public float3 delta;

    public Grid grid;

    __init(Grid volumeGrid, Ray ray) {
        valid = false;
        var aabb = AABB(volumeGrid.min, volumeGrid.max);
        var hit = aabb.intersect(ray);

        if (!hit.hit)
            return;
        entry_t = max(hit.tEntry, 0.0);
        valid = true;

        var entryPos =
            ((ray.origin + ray.direction * (entry_t + 1e-4)) - volumeGrid.min) *
            volumeGrid.voxels_per_unit();

        grid = volumeGrid;
        step = sign(ray.direction);
        delta = abs(1. / ray.direction);

        pos = int3(clamp(floor(entryPos), 0, grid.size - 1));
        tmax = (pos - entryPos + max(step, 0)) / ray.direction;

        t = 0.0;

        var tMin = min((volumeGrid.min - ray.origin) / ray.direction,
                       +(volumeGrid.max - ray.origin) / ray.direction);
        if (tMin.x >= tMin.y && tMin.x >= tMin.z)
            axis = 0;
        else if (tMin.y >= tMin.z)
            axis = 1;
        else
            axis = 2;
    }
}

public struct DDA {
    public int maxSteps;
    int steps = 0;
    bool needsStep;
    DDAState state;

    public __init(Grid volumeGrid, Ray ray, int maxSteps_ = 128) {
        maxSteps = maxSteps_;
        state = DDAState(volumeGrid, ray);
    }

    [mutating]
    public bool next(out int3 pos) {
        if (!state.valid)
            return false;

        if (needsStep && !step())
            return false;

        needsStep = true;
        pos = state.pos;
        return true;
    }

    [mutating]
    public bool step() {
        if (steps > maxSteps || !state.valid) {
            return false;
        }
        steps += 1;

        if (state.tmax.x < state.tmax.y) {
            if (state.tmax.x < state.tmax.z) {
                STEP_ALONG_AXIS(x, 0);
            } else {
                STEP_ALONG_AXIS(z, 2);
            }
        } else {
            if (state.tmax.y < state.tmax.z) {
                STEP_ALONG_AXIS(y, 1);
            } else {
                STEP_ALONG_AXIS(z, 2);
            }
        }

        return true;
    }

    public DDAHitInfo hit() {
        var vpu = state.grid.voxels_per_unit();

        var entry_t = steps <= 0 ? state.entry_t
                                 : state.entry_t + (state.tmax[state.axis] -
                                                    state.delta[state.axis]) /
                                                       vpu[state.axis];

        var exit_t =
            state.entry_t + min(state.tmax.x, min(state.tmax.y, state.tmax.z)) /
                                vpu[state.axis];

        var hitInfo = HitInfo(true, entry_t, exit_t);
        var voxelAABB = AABB(state.grid.min + state.pos / vpu,
                             state.grid.min + (state.pos + 1.) / vpu);

        return DDAHitInfo(hitInfo, voxelAABB, int3(state.pos), state.axis);
    }
}

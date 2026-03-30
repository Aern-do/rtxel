import ray;
import volume;

public struct DDAState
{
    float entryTime;
    float3 stepDirection;
    float3 delta;
    int3 pos;
    float3 tmax;
    int axis;
    uint steps;

    bool valid;
}

public struct DDAHit
{
    public float t;
    public int3 pos;
    public int axis;
    public float3 voxelMin;
    public float3 voxelMax;

    public float3 normal(float3 rayDir)
    {
        var n = float3(0.0);
        n[axis] = rayDir[axis] > 0.0 ? -1.0 : 1.0;
        return n;
    }
}

public struct DDAIterator<V : IVolume>
{
    private const float EPSILON = 1e-4;

    private V volume;
    private int maxSteps;

    private DDAState state;
    private bool needsStep;

    public __init(V volume, Ray ray, int maxSteps)
    {
        this.volume = volume;
        this.maxSteps = maxSteps;
        state.valid = false;
        state.axis = 0;
        state.steps = 0;

        var config = volume.config();

        var entryTime = rayAABB(config.min, config.max, ray);
        if (entryTime >= 1e30)
            return;

        state.entryTime = entryTime;
        var entryPos =
            ((ray.origin + ray.direction * (entryTime + EPSILON)) - config.min) * voxelsPerUnit();

        state.stepDirection = signOfDir(ray.direction);
        state.delta = abs(1.0 / ray.direction);

        state.pos = int3(
            clamp(floor(entryPos), 0.0, float(config.size) - 1.0));
        state.tmax = (float3(state.pos) - entryPos + max(state.stepDirection, 0.0)) / ray.direction;
        state.valid = true;
    }

    [mutating]
    public bool step()
    {
        if (!state.valid)
            return false;

        var config = volume.config();

        if (state.tmax.x < state.tmax.y)
        {
            if (state.tmax.x < state.tmax.z)
            {
                state.pos.x += int(state.stepDirection.x);
                if (state.pos.x < 0 || state.pos.x >= config.size)
                {
                    state.valid = false;
                    return false;
                }
                state.axis = 0;
                state.tmax.x += state.delta.x;
            }
            else
            {
                state.pos.z += int(state.stepDirection.z);
                if (state.pos.z < 0 || state.pos.z >= config.size)
                {
                    state.valid = false;
                    return false;
                }
                state.axis = 2;
                state.tmax.z += state.delta.z;
            }
        }
        else
        {
            if (state.tmax.y < state.tmax.z)
            {
                state.pos.y += int(state.stepDirection.y);
                if (state.pos.y < 0 || state.pos.y >= config.size)
                {
                    state.valid = false;
                    return false;
                }
                state.axis = 1;
                state.tmax.y += state.delta.y;
            }
            else
            {
                state.pos.z += int(state.stepDirection.z);
                if (state.pos.z < 0 || state.pos.z >= config.size)
                {
                    state.valid = false;
                    return false;
                }
                state.axis = 2;
                state.tmax.z += state.delta.z;
            }
        }

        state.steps++;
        if (state.steps >= maxSteps)
        {
            state.valid = false;
            return false;
        }

        return true;
    }

    [mutating]
    public bool next(out V.Voxel voxel)
    {
        if (needsStep && !step())
            return false;

        do
        {
            voxel = volume.get(state.pos);
            if (!volume.shouldSkip(voxel))
            {
                needsStep = true;
                return true;
            }
        } while (step());

        return false;
    }

    public DDAHit hit()
    {
        DDAHit h;
        var vpu = voxelsPerUnit();

        if (state.steps == 0)
            h.t = state.entryTime;
        else
            h.t = state.entryTime + (state.tmax[state.axis] - state.delta[state.axis]) / vpu[state.axis];

        h.pos = state.pos;
        h.axis = state.axis;

        var voxelSize = 1.0 / vpu;
        var config = volume.config();
        h.voxelMin = config.min + float3(state.pos) * voxelSize;
        h.voxelMax = h.voxelMin + voxelSize;

        return h;
    }

    public bool isValid()
    {
        return state.valid;
    }

    private float3 voxelsPerUnit()
    {
        var config = volume.config();
        return float(config.size) / (config.max - config.min);
    }

    private static float rayAABB(float3 bmin, float3 bmax, Ray ray)
    {
        var tmin = 0.0;
        var tmax = 1e30;

        [unroll]
        for (int i = 0; i < 3; ++i)
        {
            var t1 = (bmin[i] - ray.origin[i]) / ray.direction[i];
            var t2 = (bmax[i] - ray.origin[i]) / ray.direction[i];

            var dmin = min(t1, t2);
            var dmax = max(t1, t2);

            tmin = max(dmin, tmin);
            tmax = min(dmax, tmax);
        }

        return (tmax >= tmin) ? tmin : 1e30;
    }

    private static float3 signOfDir(float3 v)
    {
        return float3(
            v.x >= 0.0 ? 1.0 : -1.0,
            v.y >= 0.0 ? 1.0 : -1.0,
            v.z >= 0.0 ? 1.0 : -1.0);
    }
}

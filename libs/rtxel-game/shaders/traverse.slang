import core.ray;
import core.volume;
import core.dda;
import world;

public struct HitResult
{
    public bool didHit;
    public float t;
};

public struct Traverser
{
    public StructuredBuffer<uint> bricks;
    public StructuredBuffer<BrickMap> maps;

    public HitResult traverse(Ray ray)
    {
        var result = HitResult();

        var bricks = BrickGridVolume(bricks);
        var brickIter = DDAIterator<BrickGridVolume>(bricks, ray, 512);

        uint brick;
        while (brickIter.next(brick))
        {
            var hit = brickIter.hit();

            var brickMap = BrickMapVolume(maps[brick].mask, hit.voxelMin, hit.voxelMax);
            var brickMapIter = DDAIterator<BrickMapVolume>(brickMap, ray, 64);

            bool voxel;
            if (brickMapIter.next(voxel))
            {
                var hit = brickIter.hit();

                result.didHit = true;
                result.t = hit.t;
                return result;
            }
        }

        return result;
    }
}

public struct BrickGridVolume : IVolume
{
    StructuredBuffer<uint> buffer;

    public __init(StructuredBuffer<uint> gridBuffer)
    {
        buffer = gridBuffer;
    }

    typealias Voxel = uint;

    public VolumeConfig config()
    {
        return VolumeConfig(32, float3(-16.0 * 8.0), float3(16.0 * 8.0));
    }

    public Voxel get(int3 pos)
    {
        var config = config();

        int idx = pos.z * config.size * config.size + pos.y * config.size + pos.x;
        return buffer[idx];
    }

    public bool shouldSkip(Voxel voxel)
    {
        return voxel == GRID_EMPTY || voxel == GRID_UNLOADED;
    }
}

public struct BrickMapVolume : IVolume
{
    BrickMask mask;
    float3 bmin;
    float3 bmax;

    public __init(BrickMask mask, float3 bmin, float3 bmax)
    {
        this.mask = mask;
        this.bmin = bmin;
        this.bmax = bmax;
    }

    typealias Voxel = bool;

    public VolumeConfig config()
    {
        return VolumeConfig(8, bmin, bmax);
    }

    public Voxel get(int3 pos)
    {
        int bitIndex = pos.z * 64 + pos.y * 8 + pos.x;
        int wordIndex = bitIndex >> 5;
        int bitOffset = bitIndex & 31;
        return (mask[wordIndex] & (1u << bitOffset)) != 0;
    }

    public bool shouldSkip(Voxel voxel)
    {
        return !voxel;
    }
}

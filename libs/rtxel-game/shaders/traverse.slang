import core.ray;
import core.volume;
import core.dda;
import world;

public struct HitResult
{
    public bool didHit;
    public float3 normal;
    public float3 pos;
    public GpuMaterial material;
    public float t;
};

public struct Traverser
{
    public StructuredBuffer<uint> bricks;
    public StructuredBuffer<BrickMap> maps;
    public StructuredBuffer<GpuMaterial> materials;
    public StructuredBuffer<GpuPallete> palletes;

    public HitResult traverse(Ray ray)
    {
        var result = HitResult();

        var bricks = BrickGridVolume(bricks);
        var brickIter = DDAIterator<BrickGridVolume>(bricks, ray, 512);

        uint brick;
        while (brickIter.next(brick))
        {
            var hit = brickIter.hit();

            var brickMap = BrickMapVolume(maps[brick - 1].mask, hit.voxelMin, hit.voxelMax);
            var brickMapIter = DDAIterator<BrickMapVolume>(brickMap, ray, 128);

            bool voxel;
            if (brickMapIter.next(voxel))
            {
                var brickHit = brickMapIter.hit();
                var materialIdx = getMaterial(maps[brick - 1], brickHit.voxelPos);

                result.didHit = true;
                result.normal = brickHit.normal(ray.direction);
                result.pos = ray.origin + ray.direction * brickHit.t;
                result.material = materials[materialIdx];
                result.t = brickHit.t;
                return result;
            }
        }

        return result;
    }

    uint getMaterial(BrickMap map, int3 localPos)
    {
        uint idx = map.material_idx;

        if (!map.isPallete(idx))
        {
            return idx - 1;
        }

        int voxelIndex = localPos.z * 64 + localPos.y * 8 + localPos.x;
        return palletes[map.palleteIndex(idx)].data[voxelIndex] - 1;
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

        int idx = pos.x + pos.y * config.size + pos.z * config.size * config.size;
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

import world;
import aabb;

public struct Configuration {
    public float3 min;
    public float3 max;
    public uint3 size;

    public float3 voxels_per_unit() { return (float3)size / (max - min); }
}

public interface Volume {
    public associatedtype Voxel;
    public Configuration config();

    public Voxel get(int3 pos);
    public bool shouldSkip(Voxel voxel);
}

public struct GridVolume : Volume {
    public uint3 size;
    public StructuredBuffer<GridCell> buffer;

    public typealias Voxel = GridCell;
    public Configuration config() {
        float3 half = float3(size) * 0.5 * 8.0;

        return Configuration(-half, half, size);
    }

    public Voxel get(int3 pos) {
        var idx = flatten(pos, int3(config().size));
        return buffer[idx];
    }
    public bool shouldSkip(Voxel voxel) {
        return voxel.isUnloaded() || voxel.isEmpty();
    }
}

public struct MapVolume<let N : uint> : Volume {
    public AABB aabb;
    public Map<N> map;

    public typealias Voxel = bool;
    public Configuration config() {
        return Configuration(aabb.min_, aabb.max_, uint3(N));
    }

    public Voxel get(int3 pos) {
        var idx = flatten(uint3(pos), uint3(N));
        var word = idx / 32;
        var bit = 1u << (idx % 32);
        return (map.mask[word] & bit) != 0;
    }
    public bool shouldSkip(Voxel voxel) { return !voxel; }
}

public typealias Map8Volume = MapVolume<8>;
public typealias Map4Volume = MapVolume<4>;
public typealias Map2Volume = MapVolume<2>;

uint flatten(uint3 pos, uint3 size) {
    return pos.x + pos.y * size.x + pos.z * size.x * size.y;
}

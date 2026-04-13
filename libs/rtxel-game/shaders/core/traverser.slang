import volume;
import aabb;
import world;
import dda;
import ray;

uint flatten(int3 pos, int size) {
    return pos.x + pos.y * size + pos.z * size * size;
}

public struct VoxelHitInfo {
    public HitInfo hitInfo;
    public float3 normal;
    public float3 tint = 1.0;
    public int3 pos;
    public Material material;

    public bool didHit() { return hitInfo.hit; }
}

public struct GridConfiguration {
    public int size;
}

public struct BrickGridVolume : Volume {
    public GridConfiguration configuration;
    public StructuredBuffer<uint> buffer;
    public typealias Voxel = uint;

    public Grid grid() {
        var halfSize = float3(configuration.size) / 2.;
        return Grid(-halfSize * 8.0, halfSize * 8.0, configuration.size);
    }

    public Voxel get(int3 pos) {
        var idx = flatten(pos, grid().size);
        return buffer[idx];
    }

    public bool shouldSkip(Voxel voxel) {
        return voxel == GRID_EMPTY || voxel == GRID_UNLOADED;
    }
}

public struct BrickMapVolume : Volume {
    public AABB aabb;
    public BrickMask mask;

    public typealias Voxel = bool;

    public Grid grid() { return Grid(aabb.min_, aabb.max_, 8); }

    public Voxel get(int3 pos) {
        var index = flatten(pos, grid().size);

        var word = index / 32;
        var bit = 1 << (index % 32);

        return (mask[word] & bit) != 0;
    }

    public bool shouldSkip(Voxel voxel) { return !voxel; }
}

public struct Traverser {
    public BrickGridVolume gridVolume;
    public StructuredBuffer<BrickMap> bricks;
    public StructuredBuffer<Material> materials;
    public StructuredBuffer<Pallete> palletes;

    public VoxelHitInfo traverse(Ray ray, int maxGridSteps = 128, int maxMapSteps = 64) {
        var result = VoxelHitInfo();

        var grid = gridVolume.grid();
        var gridDDA = DDA(grid, ray, maxGridSteps);

        int3 gridPos;
        while (gridDDA.next(gridPos)) {
            var brick = gridVolume.get(gridPos);
            if (gridVolume.shouldSkip(brick))
                continue;

            var hit = gridDDA.hit();
            var map = BrickMapVolume(hit.voxelAABB, bricks[brick - 1].mask);
            var mapDDA = DDA(map.grid(), ray, maxMapSteps);

            int3 innerPos;
            while (mapDDA.next(innerPos)) {
                var voxel = map.get(innerPos);
                if (map.shouldSkip(voxel))
                    continue;

                var innerHit = mapDDA.hit();
                var materialIdx = getMaterial(bricks[brick - 1], innerPos);
                var material = materials[materialIdx];

                if (material.transparency > 0.0) {
                    result.tint *= material.color;
                    continue;
                }

                result.hitInfo = innerHit.hitInfo;
                result.normal = innerHit.normal(ray);
                result.material = material;
                result.pos = int3(innerHit.voxelAABB.min_);
                return result;
            }
        }

        return result;
    }

    uint getMaterial(BrickMap map, int3 pos) {
        if (!map.isPallete(map.material_idx)) {
            return map.material_idx - 1;
        }

        var idx = flatten(pos, 8);
        var pallete = palletes[map.palleteIndex(map.material_idx)];
        return pallete[idx] - 1;
    }
}

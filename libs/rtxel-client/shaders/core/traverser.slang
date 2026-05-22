import volume;
import aabb;
import world;
import dda;
import ray;
import camera;

public struct VoxelHitInfo {
    public HitInfo hitInfo;
    public float3 normal;
    public Lod lod;
    public AABB aabb;
    public bool didHit() { return hitInfo.hit; }
}

public enum Lod {
    Map8,
    Map4,
    Map2,
}

public extension Lod {
    public int size() {
        switch (this) {
        case Lod.Map8:
            return 8;
        case Lod.Map4:
            return 4;
        case Lod.Map2:
            return 2;
        default:
            return 8;
        }
    }
}

Lod selectLod(float dist) {
    float d = dist / Map8::SIZE;

    if (d < 32.0)
        return Lod.Map8;
    if (d < 48.0)
        return Lod.Map4;
    return Lod.Map2;
}

public struct Traverser {
    public Camera camera;
    public GridVolume gridVolume;
    public StructuredBuffer<Map8> map8Buffer;
    public StructuredBuffer<Map4> map4Buffer;
    public StructuredBuffer<Map2> map2Buffer;

    void traverseMap<let N : uint>(Ray ray, MapVolume<N> map,
                                   inout VoxelHitInfo result) {
        ray.origin += ray.direction * (map.config().size.x / N) * 1e-3;
        var dda = DDA(map.config(), ray, int(N * N * N));
        int3 pos;

        while (dda.next(pos)) {
            var voxel = map.get(pos);
            if (map.shouldSkip(voxel))
                continue;

            var hit = dda.hit();

            result.hitInfo = hit.hitInfo;
            result.aabb = hit.voxelAABB;
            result.normal = hit.normal(ray);
            return;
        }
    }

    public VoxelHitInfo traverse(Ray ray) {
        var result = VoxelHitInfo();
        var gridConfig = gridVolume.config();

        var gridDDA = DDA(gridConfig, ray, dot(gridConfig.size, 1));

        int3 gridPos;
        while (gridDDA.next(gridPos)) {
            var cell = gridVolume.get(gridPos);
            if (gridVolume.shouldSkip(cell))
                continue;

            var hit = gridDDA.hit();

            var brickCenter = (hit.voxelAABB.min_ + hit.voxelAABB.max_) * 0.5;
            var dist = length(brickCenter - camera.origin);
            var lod = selectLod(dist);

            switch (lod) {
            case Lod.Map8: {
                var map = Map8Volume(hit.voxelAABB, map8Buffer[cell.ptr]);
                traverseMap<8>(ray, map, result);
                break;
            }
            case Lod.Map4: {
                var map = Map4Volume(hit.voxelAABB, map4Buffer[cell.ptr]);
                traverseMap<4>(ray, map, result);

                break;
            }
            case Lod.Map2: {
                var map = Map2Volume(hit.voxelAABB, map2Buffer[cell.ptr]);
                traverseMap<2>(ray, map, result);
                break;
            }
            }

            if (result.didHit())
                return result;
        }
        return result;
    }
}

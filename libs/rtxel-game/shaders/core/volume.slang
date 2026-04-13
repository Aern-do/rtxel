public struct Grid {
    public float3 min;
    public float3 max;
    public int size;

    public float3 voxels_per_unit() { return size / (max - min); }
}

public interface Volume {
    public associatedtype Voxel;
    public Grid grid();

    public Voxel get(int3 pos);
    public bool shouldSkip(Voxel voxel);
}

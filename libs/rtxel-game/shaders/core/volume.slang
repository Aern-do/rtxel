public struct VolumeConfig {
    public int size;
    public float3 min;
    public float3 max;
};

public interface IVolume {
    associatedtype Voxel;

    public VolumeConfig config();
    
    public Voxel get(int3 pos);
    public bool shouldSkip(Voxel voxel);
}
import core.volume;
import core.dda;
import ray;

public typealias BrickMask = Array<uint, 16>;
public static const uint MAP_NOT_REQUESTED = 0;
public static const uint MAP_REQUESTED = 1;

public static const uint GRID_UNLOADED = 0xFFFF;
public static const uint GRID_EMPTY = 0;

public struct BrickMap
{
    public static const uint MATERIAL_PALLETE_BIT = 0x80000000u;

    public BrickMask mask;
    public uint material_idx;
    public uint is_requested;

    public bool isPallete(uint material_idx)
    {
        return (material_idx & MATERIAL_PALLETE_BIT) != 0;
    }

    public uint palleteIndex(uint material_idx)
    {
        return material_idx & ~MATERIAL_PALLETE_BIT;
    }
}

public struct GpuPallete {
    public Array<uint, 512> data;
}

public struct GpuMaterial {
    public float3 color;
}
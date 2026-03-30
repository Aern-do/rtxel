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
    public BrickMask mask;
    public uint is_requested;
}



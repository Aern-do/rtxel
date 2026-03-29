public typealias BrickMask = Array<uint, 16>;
public static uint MAP_NOT_REQUESTED = 0;
public static uint MAP_REQUESTED = 1;

public static uint GRID_UNLOADED = 0xFFFF;
public static uint GRID_EMPTY = 0xFFFF;

public struct BrickMap {
    public BrickMask mask;
    public uint is_requested;
}
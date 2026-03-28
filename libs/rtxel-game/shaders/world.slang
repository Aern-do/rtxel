public typealias BrickMask = Array<uint, 16>;
public static uint MAP_NOT_REQUESTED = 0;
public static uint MAP_REQUESTED = 1;

public struct BrickMap {
    public BrickMask mask;
    public uint is_requested;
}
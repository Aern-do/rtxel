public struct Map<let N : uint> {
    public static const uint SIZE = N;
    public static const uint VOLUME = N * N * N;
    public static const uint WORDS = (N * N * N + 31) / 32;

    public typealias Mask = Array<uint, WORDS>;

    public Mask mask;
}

public typealias Map8 = Map<8>;
public typealias Map4 = Map<4>;
public typealias Map2 = Map<2>;

public struct GridCell {
    uint raw;
    __init(uint value) { raw = value; }
    public static GridCell unloaded() { return GridCell(0); }
    public static GridCell empty() { return GridCell(1); }
    public static GridCell map(uint ptr, uint height) {
        return GridCell((height << 28) | (ptr + 2));
    }
    public bool isUnloaded() { return (raw & 0x0FFFFFFF) == 0; }
    public bool isEmpty() { return (raw & 0x0FFFFFFF) == 1; }
    public bool isMap() { return (raw & 0x0FFFFFFF) >= 2; }

    public property uint ptr {
        get { return (raw & 0x0FFFFFFF) - 2; }
    }
    public property uint height {
        get { return raw >> 28; }
    }
}

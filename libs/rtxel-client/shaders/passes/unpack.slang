import core.world;

enum Action : uint { Clear, Set }

struct Command {
    Action action;
    uint grid;
    uint map;

    Map2 map2;
    Map4 map4;
    Map8 map8;
}

struct Size {
    uint value;
}

struct UnpackGroup {
    StructuredBuffer<Command> commandBuffer;
    ConstantBuffer<Size> commandSizeBuffer;

    RWStructuredBuffer<GridCell> gridBuffer;
    RWStructuredBuffer<Map8> map8Buffer;
    RWStructuredBuffer<Map4> map4Buffer;
    RWStructuredBuffer<Map2> map2Buffer;
}

ParameterBlock<UnpackGroup> bindings;

[shader("compute")]
[numthreads(64, 1, 1)]
void main(uint3 threadId: SV_DispatchThreadID) {
    uint size = bindings.commandSizeBuffer.value;
    if (threadId.x >= size)
        return;

    Command cmd = bindings.commandBuffer[threadId.x];
    if (cmd.action == Action.Set) {
        bindings.gridBuffer[cmd.grid] = GridCell.map(cmd.map);
        bindings.map8Buffer[cmd.map] = cmd.map8;
        bindings.map4Buffer[cmd.map] = cmd.map4;
        bindings.map2Buffer[cmd.map] = cmd.map2;
    } else if (cmd.action == Action.Clear) {
        bindings.gridBuffer[cmd.grid] = GridCell.empty();
    }
}

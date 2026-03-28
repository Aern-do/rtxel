import world;

struct UnpackCommand {
    uint grid_idx;
    uint map_idx;
    uint op;
    Array<uint, 16> data;
}

static uint UNPACK_OP_CLEAR = 0;
static uint UNPACK_OP_UNLOAD = 1;
static uint UNPACK_OP_SET = 2;

[vk::binding(0, 0)]
StructuredBuffer<UnpackCommand> commands;
[vk::binding(1, 0)]
ConstantBuffer<uint> commands_len;

[vk::binding(0, 1)]
RWStructuredBuffer<uint> grid;
[vk::binding(1, 1)]
RWStructuredBuffer<BrickMap> map;

[shader("compute")]
[numthreads(64)]
void main(uint3 threadId : SV_DispatchThreadID) {
    uint idx = threadId.x;
    if (idx >= commands_len) {
        return;
    }

    UnpackCommand cmd = commands[idx];

    if (cmd.op == UNPACK_OP_CLEAR) {
        map[cmd.map_idx] = {};
        grid[cmd.grid_idx] = 0xFFFF;
    } else if (cmd.op == UNPACK_OP_UNLOAD) {
        grid[cmd.grid_idx] = 0;
    } else if (cmd.op == UNPACK_OP_SET) {
        grid[cmd.grid_idx] = cmd.map_idx + 1;
        BrickMap entry;
        for (uint i = 0; i < 16; i++) {
            entry.mask[i] = cmd.data[i];
        }

        map[cmd.map_idx] = entry;
    }
}
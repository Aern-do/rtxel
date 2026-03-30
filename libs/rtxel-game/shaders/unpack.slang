import world;

struct UnpackCommand
{
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
void main(uint3 threadId: SV_DispatchThreadID)
{
    var idx = threadId.x;
    if (idx >= commands_len)
    {
        return;
    }

    var cmd = commands[idx];

    if (cmd.op == UNPACK_OP_CLEAR)
    {
        map[cmd.map_idx] = {};
        grid[cmd.grid_idx] = GRID_EMPTY;
    }
    else if (cmd.op == UNPACK_OP_UNLOAD)
    {
        grid[cmd.grid_idx] = GRID_UNLOADED;
    }
    else if (cmd.op == UNPACK_OP_SET)
    {
        BrickMap entry = {};
        entry.is_requested = false;
        for (uint i = 0; i < 16; i++)
        {
            entry.mask[i] = cmd.data[i];
        }

        map[cmd.map_idx] = entry;
        grid[cmd.grid_idx] = cmd.map_idx + 1;
    }
}

import world;

static const uint OP_CLEAR = 0;
static const uint OP_SET_UNIFORM = 1;
static const uint OP_SET_PALLETE = 2;

[vk::binding(0, 0)]
StructuredBuffer<uint> command_data;

[vk::binding(1, 0)]
ConstantBuffer<uint> command_count;

[vk::binding(2, 0)]
StructuredBuffer<uint> command_offsets;

[vk::binding(3, 0)]
RWStructuredBuffer<BrickMap> map;

[vk::binding(4, 0)]
RWStructuredBuffer<uint> grid;

[vk::binding(5, 0)]
RWStructuredBuffer<GpuPallete> palletes;

[shader("compute")]
[numthreads(64, 1, 1)]
void main(uint3 threadId : SV_DispatchThreadID)
{
    uint idx = threadId.x;
    if (idx >= command_count)
    {
        return;
    }

    uint offset = command_offsets[idx];
    uint op = command_data[offset + 0];
    uint grid_idx = command_data[offset + 1];

    if (op == OP_CLEAR)
    {
        grid[grid_idx] = GRID_EMPTY;
    }
    else if (op == OP_SET_UNIFORM)
    {
        uint map_idx = command_data[offset + 2];
        uint material_idx = command_data[offset + 3];

        BrickMap entry = {};
        entry.is_requested = false;
        entry.material_idx = material_idx;
        for (uint i = 0; i < 16; i++)
            entry.mask[i] = command_data[offset + 4 + i];

        map[map_idx] = entry;
        grid[grid_idx] = map_idx + 1;
    }
    else if (op == OP_SET_PALLETE)
    {
        uint map_idx = command_data[offset + 2];
        uint pallete_idx = command_data[offset + 3];

        BrickMap entry = {};
        entry.is_requested = false;
        entry.material_idx = pallete_idx | BrickMap::MATERIAL_PALLETE_BIT;
        for (uint i = 0; i < 16; i++)
            entry.mask[i] = command_data[offset + 4 + i];

        GpuPallete pallete = {};
        for (uint i = 0; i < 512; i++)
            pallete.data[i] = command_data[offset + 20 + i];

        palletes[pallete_idx] = pallete;
        map[map_idx] = entry;
        grid[grid_idx] = map_idx + 1;
    }
}
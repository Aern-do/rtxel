import world;
import traverse;
import core.camera;
import core.ray;
import core.dda;

[vk::binding(0, 0)]
ConstantBuffer<Camera> camera;

[vk::binding(1, 0)]
WTexture2D<float4> outputTexture;

[vk::binding(2, 0)]
StructuredBuffer<uint> bricks;
[vk::binding(3, 0)]
StructuredBuffer<BrickMap> maps;

[vk::binding(4, 0)]
StructuredBuffer<GpuPallete> palletes;
[vk::binding(5, 0)]
StructuredBuffer<GpuMaterial> materials;

[shader("compute")]
[numthreads(8, 8)]
void main(uint3 threadId: SV_DispatchThreadID)
{
    uint width, height;
    outputTexture.GetDimensions(width, height);

    if (threadId.x >= width || threadId.y >= height)
        return;

    var uv = (float2(threadId.xy) + 0.5) / float2(width, height);
    var ray = camera.createRay(uv);

    var color = float3(0.0, 0.0, 0.4);

    var traverser = Traverser(bricks, maps, materials, palletes);
    var hit = traverser.traverse(ray);

    if (hit.didHit) {
        color = hit.material.color;
    }

    outputTexture.Store(threadId.xy, float4(color, 1.0));
}

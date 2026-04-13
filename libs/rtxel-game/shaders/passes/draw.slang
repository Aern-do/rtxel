import core.camera;
import core.world;
import core.traverser;
import core.ray;

[vk::binding(0, 0)]
ConstantBuffer<Camera> camera;
[vk::binding(1, 0)]
ConstantBuffer<GridConfiguration> configuration;

[vk::binding(2, 0)]
WTexture2D<float4> outputTexture;

[vk::binding(3, 0)]
StructuredBuffer<uint> grid;
[vk::binding(4, 0)]
StructuredBuffer<BrickMap> bricks;

[vk::binding(5, 0)]
StructuredBuffer<Pallete> palletes;
[vk::binding(6, 0)]
StructuredBuffer<Material> materials;

[shader("compute")]
[numthreads(8, 8)]
void main(uint3 threadId: SV_DispatchThreadID) {
    uint width, height;
    outputTexture.GetDimensions(width, height);

    if (threadId.x >= width || threadId.y >= height)
        return;

    var uv = (float2(threadId.xy) + 0.5) / float2(width, height);
    var ray = camera.createRay(uv);

    var color = traverse(ray);
    outputTexture.Store(threadId.xy, float4(color, 1.0));
}

float3 traverse(Ray ray) {
    var gridVolume = BrickGridVolume(configuration, grid);
    var traverser = Traverser(gridVolume, bricks, materials, palletes);

    var hit = traverser.traverse(ray);
    if (hit.didHit()) {
        return hit.material.color;
    }

    return float3(0.0);
}

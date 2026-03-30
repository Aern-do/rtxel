import world;
import traverse;
import core.camera;
import core.ray;
import core.dda;

[vk::binding(0, 0)]
StructuredBuffer<uint> bricks;
[vk::binding(1, 0)]
StructuredBuffer<BrickMap> maps;
[vk::binding(2, 0)]
WTexture2D<float4> outputTexture;

[vk::binding(0, 1)]
ConstantBuffer<Camera> camera;

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

    var traverser = Traverser(bricks, maps);
    var hit = traverser.traverse(ray);
    if (hit.didHit) {
        color = float3(hit.t / 1024);
    }

    outputTexture.Store(threadId.xy, float4(color, 1.0));
}

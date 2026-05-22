import core.world;
import core.camera;
import core.ray;
import core.traverser;
import core.pathTracer;
import core.volume;
import core.rng;

struct RenderData {
    uint3 gridSize;
    uint frame;
}

struct DrawGroup {
    ConstantBuffer<Camera> cameraBuffer;
    ConstantBuffer<RenderData> renderDataBuffer;
    Texture2D<float4> texture1;
    WTexture2D<float4> texture2;
    StructuredBuffer<GridCell> gridBuffer;
    StructuredBuffer<Map8> map8Buffer;
    StructuredBuffer<Map4> map4Buffer;
    StructuredBuffer<Map2> map2Buffer;
}
ParameterBlock<DrawGroup> bindings;

[shader("compute")]
[numthreads(8, 8, 1)]
void main(uint3 threadId: SV_DispatchThreadID) {
    uint width, height;
    bindings.texture1.GetDimensions(width, height);
    if (threadId.x >= width || threadId.y >= height)
        return;

    var camera = bindings.cameraBuffer;
    var renderData = bindings.renderDataBuffer;
    var rng =
        Rng(threadId.x + threadId.y * width + camera.frame * width * height);

    var uv = (float2(threadId.xy) + 0.5) / float2(width, height);
    var ray = camera.createRay(uv);

    var gridVolume = GridVolume(renderData.gridSize, bindings.gridBuffer);
    var traverser = Traverser(camera, gridVolume, bindings.map8Buffer,
                              bindings.map4Buffer, bindings.map2Buffer);

    var pathTracer = PathTracer(rng, traverser);
    var sample = pathTracer.trace(ray);
    var accumulated = float3(0.);

    if (camera.frame == 0) {
        accumulated = sample;
    } else {
        float3 prev = bindings.texture1.Load(int3(threadId.xy, 0)).rgb;
        float n = float(camera.frame);
        accumulated = prev + (sample - prev) / (n + 1.0);
    }

    bindings.texture2.Store(threadId.xy, float4(accumulated, 1.0));
}

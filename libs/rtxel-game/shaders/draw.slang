import world;

struct Camera {
    float3 origin;
    float yaw;
    float pitch;

    float3 forward;
    float3 up;
    float aspect;
    float fov;

    uint frame_count;
};

[vk::binding(0, 0)]
RWStructuredBuffer<uint> grid;
[vk::binding(1, 0)]
RWStructuredBuffer<BrickMap> map;
[vk::binding(2, 0)]
WTexture2D<float4> outputTexture;

[vk::binding(0, 1)]
ConstantBuffer<Camera> camera;

[shader("compute")]
[numthreads(8, 8)]
void main(uint3 threadId : SV_DispatchThreadID) {
    uint width, height;
    outputTexture.GetDimensions(width, height);

    if (threadId.x >= width || threadId.y >= height)
        return;

    float2 uv = float2(threadId.xy) / float2(width, height);

    float3 tint = abs(camera.forward) * 0.5 + 0.5;
    float3 color = float3(uv.x, uv.y, 0.5) * tint;

    outputTexture.Store(threadId.xy, float4(color, 1.0));
}
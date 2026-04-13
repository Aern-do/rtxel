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

    float angle = float(camera.frame_count) * 0.00025;
    var sun = normalize(float3(cos(angle), 0.4, sin(angle)));

    var hit = traverser.traverse(ray);
    if (hit.didHit()) {
        var sunRay = Ray(hit.hitInfo.entry(ray) + 1e-3, sun);
        var sunHit = traverser.traverse(sunRay);

        float highlight =
            (cos(float(camera.frame_count) * 0.05) * 0.5 + 0.5) * 0.3;
        var color = hit.material.color;
        if (all(hit.pos == camera.target_block)) {
            color += highlight;
        }

        if (sunHit.didHit()) {
            return color * 0.25 * hit.tint;
        } else {
            return color * hit.tint;
        }
    } else {
        var sunColor = smoothstep(
            0.975, 0.985, dot(normalize(ray.direction), normalize(sun)));
        var skyStartColor = float3(0.2, 0.6, 1.0);
        var skyEndColor = float3(0.0, 0.5, 1.0);

        var skyColor = lerp(skyStartColor, skyEndColor, ray.direction.y);

        var result = skyColor + sunColor;
        return result / (result + float3(1.0));
    }

    return float3(0.0);
}

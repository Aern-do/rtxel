struct CameraData {
    float3 origin;
    float3 forward;
    float3 up;
    float3 right;
    float fov;
    float aspect;
};

[format("rgba8")]
WTexture2D output;

ConstantBuffer<CameraData> camera;

struct Ray {
    float3 origin;
    float3 direction;
};

Ray generate_ray(uint2 pixel, uint width, uint height) {
    float2 uv = float2(
        (float(pixel.x) + 0.5) / float(width),
        (float(pixel.y) + 0.5) / float(height)
    );

    float2 ndc = uv * 2.0 - 1.0;
    ndc.y = -ndc.y;

    float half_h = tan(radians(camera.fov * 0.5));
    float half_w = half_h * camera.aspect;

    float3 dir = normalize(
        camera.forward
        + ndc.x * half_w * camera.right
        + ndc.y * half_h * camera.up
    );

    Ray ray;
    ray.origin = camera.origin;
    ray.direction = dir;
    return ray;
}

float3 sky_color(float3 dir) {
    float t = 0.5 * (dir.y + 1.0);
    return lerp(float3(1.0), float3(0.5, 0.7, 1.0), t);
}

bool hit_ground(Ray ray, out float t, out float3 normal) {
    if (abs(ray.direction.y) < 1e-6) return false;
    t = -ray.origin.y / ray.direction.y;
    if (t < 0.0) return false;
    normal = float3(0.0, 1.0, 0.0);
    return true;
}

bool hit_sphere(Ray ray, float3 center, float radius, out float t) {
    float3 oc = ray.origin - center;
    float a = dot(ray.direction, ray.direction);
    float b = dot(oc, ray.direction);
    float c = dot(oc, oc) - radius * radius;
    float disc = b * b - a * c;
    if (disc < 0.0) return false;
    t = (-b - sqrt(disc)) / a;
    return t > 0.0;
}

float3 trace(Ray ray) {
    float t_ground;
    float3 n_ground;
    float t_sphere;

    float closest = 1e30;
    float3 color = sky_color(ray.direction);

    if (hit_ground(ray, t_ground, n_ground) && t_ground < closest) {
        closest = t_ground;
        color = float3(1.0, 0.0, 0.0);
    }

    if (hit_sphere(ray, float3(0.0, 1.0, 3.0), 1.0, t_sphere)
        && t_sphere < closest)
    {
        closest = t_sphere;
        color = float3(0.9, 0.2, 0.15);
    }

    return color;
}

[shader("compute")]
[numthreads(8, 8)]
void main(uint3 threadId : SV_DispatchThreadID) {
    uint width = 0;
    uint height = 0;
    output.GetDimensions(width, height);

    if (threadId.x >= width || threadId.y >= height) {
        return;
    }

    Ray ray = generate_ray(threadId.xy, width, height);
    float3 color = trace(ray);

    output.Store(threadId.xy, float4(color, 1.0));
}
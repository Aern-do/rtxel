import ray;
import traverser;
import rng;

public struct PathTracer {
    public static const float EPSILON = 1e-3;
    public static const uint MAX_BOUNCES = 3;
    public static const float3 SKY_COLOR = float3(0.5, 0.7, 1.0);

    public float3 throughput = float3(1.);
    public float3 radiance = float3(0.);

    Rng rng;
    Traverser traverser;

    public __init(Rng rng_, Traverser traverser_) {
        rng = rng_;
        traverser = traverser_;
    }

    [mutating]
    public float3 trace(Ray ray) {
        for (var i = 0; i < MAX_BOUNCES; i++) {
            var hit = traverser.traverse(ray);
            if (!hit.didHit()) {
                radiance += throughput * SKY_COLOR;
                break;
            }

            var material = Material::temporary();
            radiance += throughput * material.emission;

            throughput *= material.albedo;

            ray.origin = hit.hitInfo.entry(ray) + hit.normal * EPSILON;
            ray.direction = cosineWeightedHemisphere(hit.normal);
        }

        return radiance;
    }

    float3 tangentToWorld(float3 localDir, float3 normal) {
        if (normal.z < -0.9999f) {
            float3 T = float3(0.0f, -1.0f, 0.0f);
            float3 B = float3(-1.0f, 0.0f, 0.0f);
            return localDir.x * T + localDir.y * B + localDir.z * normal;
        } else {
            float a = 1.0f / (1.0f + normal.z);
            float b = -normal.x * normal.y * a;
            float3 T = float3(1.0f - normal.x * normal.x * a, b, -normal.x);
            float3 B = float3(b, 1.0f - normal.y * normal.y * a, -normal.y);
            return localDir.x * T + localDir.y * B + localDir.z * normal;
        }
    }

    [mutating]
    float3 cosineWeightedHemisphere(float3 normal) {
        float r1 = rng.getRandomFloat(0., 1.);
        float r2 = rng.getRandomFloat(0., 1.);

        float phi = 2.0f * 3.14159265359f * r1;
        float cosTheta = sqrt(1.0f - r2);
        float sinTheta = sqrt(r2);

        float3 localDir =
            float3(cos(phi) * sinTheta, sin(phi) * sinTheta, cosTheta);

        return tangentToWorld(localDir, normal);
    }
}

public struct Material {
    public float3 albedo;
    public float3 emission;

    // temporary default material until actual materials are implemented
    public static Material temporary() {
        return Material(float3(0.8), float3(0.0));
    }
}

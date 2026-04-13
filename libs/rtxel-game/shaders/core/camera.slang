import ray;

public struct Camera {
  public float3 origin;
  public float yaw;
  public float pitch;

  public float3 forward;
  public float3 up;
  public float aspect;
  public float fov;

  public uint frame_count;

  public Ray createRay(float2 uv) {
    float2 ndc = float2(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);

    float3 right = normalize(cross(this.up, this.forward));
    float3 up = normalize(cross(this.forward, right));

    float halfFov = tan(radians(this.fov) * 0.5);

    Ray ray;
    ray.origin = this.origin;
    ray.direction =
        normalize(this.forward + right * (ndc.x * halfFov * this.aspect) +
                  up * (ndc.y * halfFov));
    return ray;
  }
};

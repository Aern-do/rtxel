public struct Ray {
  public float3 origin;
  public float3 direction;

  public float3 pos(float t) { return origin + (direction * t); }
}

public struct HitInfo {
  public bool hit = false;
  public float tEntry;
  public float tExit;

  public static HitInfo MISS = HitInfo(false);

  public float3 entry(Ray ray) { return ray.pos(tEntry); }
  public float3 exit(Ray ray) { return ray.pos(tExit); }
  public float lowestY(Ray ray) { return min(entry(ray).y, exit(ray).y); }
}

public interface Intersect {
  HitInfo intersect(Ray ray);
}

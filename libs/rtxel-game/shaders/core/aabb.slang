import ray;

public struct AABB : Intersect {
  public float3 min_;
  public float3 max_;

  HitInfo intersect(Ray ray) {
    var invDir = 1.0 / ray.direction;

    var tMin = (min_ - ray.origin) * invDir;
    var tMax = (max_ - ray.origin) * invDir;

    var t1 = min(tMin, tMax);
    var t2 = max(tMin, tMax);

    var tEntry = max(t1.x, max(t1.y, t1.z));
    var tExit = min(t2.x, min(t2.y, t2.z));

    if (tEntry > tExit || tExit < 0.0) {
      return HitInfo.MISS;
    }

    return HitInfo(true, tEntry, tExit);
  }
}

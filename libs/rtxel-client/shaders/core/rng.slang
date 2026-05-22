// https://gamedev.stackexchange.com/a/32715
static const int RANDOM_IA = 16807;
static const int RANDOM_IM = 2147483647;
static const float RANDOM_AM = 1.0f / float(RANDOM_IM);
static const uint RANDOM_IQ = 127773u;
static const int RANDOM_IR = 2836;
static const int RANDOM_MASK = 123459876;

uint pcgHash(uint x) {
    uint state = x * 747796405u + 2891336453u;
    uint word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

public struct Rng {
    public int seed;
    public __init(uint value) { setSeed(value); }

    [mutating]
    public void cycle() {
        seed ^= RANDOM_MASK;
        int k = seed / RANDOM_IQ;
        seed = RANDOM_IA * (seed - k * RANDOM_IQ) - RANDOM_IR * k;

        if (seed < 0)
            seed += RANDOM_IM;

        seed ^= RANDOM_MASK;
    }

    [mutating]
    public float getCurrentFloat() {
        cycle();
        return RANDOM_AM * seed;
    }

    [mutating]
    public int getCurrentInt() {
        cycle();
        return seed;
    }

    [mutating]
    public float getRandomFloat(float low, float high) {
        float v = getCurrentFloat();
        return low * (1.0f - v) + high * v;
    }

    [mutating]
    public void setSeed(uint value) {
        seed = int(pcgHash(value));
        cycle();
    }
};

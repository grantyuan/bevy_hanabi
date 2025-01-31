struct Particle {
    pos: vec3<f32>;
    age: f32;
    vel: vec3<f32>;
    lifetime: f32;
};

struct ParticleAppearArea {    
    // position: vec3<f32>; 

    // cube_vertex_min:vec3<f32>;
    // cube_vertex_max:vec3<f32>;
    position: vec3<f32>; 
    actived:i32;
    flow_direction:vec3<f32>;
    flow_speed:f32;
};

struct ParticleAppearAreaBuffer{
    // particleAppearAreas:[[stride(32)]] array<ParticleAppearArea>;

    particleAppearAreas:[[stride(32)]] array<ParticleAppearArea>;
};

struct ParticleBuffer {
    particles: [[stride(32)]] array<Particle>;
};

struct SimParams {
    dt: f32;
    time: f32;
    box_width:u32;
    box_height:u32;
    box_long:u32;
};

struct ForceFieldParam {
    position: vec3<f32>;
    max_radius: f32;
    min_radius: f32;
    mass: f32;
    force_exponent: f32;
    conform_to_sphere: f32;
};

struct Spawner {
    origin: vec3<f32>;
    spawn: atomic<i32>;
    accel: vec3<f32>;
    count: atomic<i32>;
    force_field: array<ForceFieldParam, 16>;
    __pad0: vec3<f32>;
    seed: u32;
    __pad1: vec3<f32>;    
    live_time:f32;
};

struct IndirectBuffer {
    indices: [[stride(4)]] array<u32>;
};

// var<uniform> values:array<array<array<i32>>>;

[[group(0), binding(0)]] var<uniform> sim_params : SimParams;
[[group(1), binding(0)]] var<storage, read_write> particle_buffer : ParticleBuffer;
[[group(2), binding(0)]] var<storage, read_write> spawner : Spawner;
[[group(3), binding(0)]] var<storage, read_write> indirect_buffer : IndirectBuffer;
[[group(4), binding(0)]] var<storage, read> appear_area_buffer : ParticleAppearAreaBuffer;

var<private> seed : u32 = 0u;

let tau: f32 = 6.283185307179586476925286766559;

// Rand: PCG
// https://www.reedbeta.com/blog/hash-functions-for-gpu-rendering/
fn pcg_hash(input: u32) -> u32 {
    var state: u32 = input * 747796405u + 2891336453u;
    var word: u32 = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn to_float01(u: u32) -> f32 {
    // Note: could generate only 24 bits of randomness
    return bitcast<f32>((u & 0x007fffffu) | 0x3f800000u) - 1.;
}

// Random floating-point number in [0:1]
fn rand() -> f32 {
    seed = pcg_hash(seed);
    return to_float01(pcg_hash(seed));
}

// Random floating-point number in [0:1]^2
fn rand2() -> vec2<f32> {
    seed = pcg_hash(seed);
    var x = to_float01(seed);
    seed = pcg_hash(seed);
    var y = to_float01(seed);
    return vec2<f32>(x, y);
}

// Random floating-point number in [0:1]^3
fn rand3() -> vec3<f32> {
    seed = pcg_hash(seed);
    var x = to_float01(seed);
    seed = pcg_hash(seed);
    var y = to_float01(seed);
    seed = pcg_hash(seed);
    var z = to_float01(seed);
    return vec3<f32>(x, y, z);
}

// Random floating-point number in [0:1]^4
fn rand4(input: u32) -> vec4<f32> {
    // Each rand() produces 32 bits, and we need 24 bits per component,
    // so can get away with only 3 calls.
    var r0 = pcg_hash(seed);
    var r1 = pcg_hash(r0);
    var r2 = pcg_hash(r1);
    seed = r2;
    var x = to_float01(r0);
    var r01 = (r0 & 0xff000000u) >> 8u | (r1 & 0x0000ffffu);
    var y = to_float01(r01);
    var r12 = (r1 & 0xffff0000u) >> 8u | (r2 & 0x000000ffu);
    var z = to_float01(r12);
    var r22 = r2 >> 8u;
    var w = to_float01(r22);
    return vec4<f32>(x, y, z, w);
}

struct PosVel {
    pos: vec3<f32>;
    vel: vec3<f32>;
};

fn init_pos_vel(index: u32) -> PosVel {
    var ret : PosVel;
    var speed: f32 = appear_area_buffer.particleAppearAreas[index].flow_speed;

{{INIT_POS_VEL}}

    return ret;
}

fn init_lifetime() -> f32 {
    return spawner.live_time;
}

fn proj(u: vec3<f32>, v: vec3<f32>) -> vec3<f32> {
    return dot(v, u) / dot(u,u) * u;
}

// var<private> my_test_value:f32;

[[stage(compute), workgroup_size(64)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    let max_particles : u32 = arrayLength(&particle_buffer.particles);
    let max_appear_areas : u32 = arrayLength(&appear_area_buffer.particleAppearAreas);
    
    if (max_appear_areas == u32(0)) {
        return;
    }

    if (max_appear_areas == u32(1) && appear_area_buffer.particleAppearAreas[0].actived == i32(-1))
    {
        return;
    }

    let index = global_invocation_id.x;
    if (index >= max_particles) {
        return;
    }
    let appear_area_index = index%max_appear_areas;
    var vPos : vec3<f32> = particle_buffer.particles[index].pos;
    var vVel : vec3<f32> = particle_buffer.particles[index].vel;
    var vAge : f32 = particle_buffer.particles[index].age;
    var vLifetime : f32 = particle_buffer.particles[index].lifetime;
    var appear_area_pos: vec3<f32> = appear_area_buffer.particleAppearAreas[appear_area_index].position;
    var direction: vec3<f32> = appear_area_buffer.particleAppearAreas[appear_area_index].flow_direction;
    var speed: f32 = appear_area_buffer.particleAppearAreas[appear_area_index].flow_speed;
    // Age the particle
    vAge = vAge + sim_params.dt;
    if (vAge >= vLifetime) {
        // Particle dead; try to recycle into newly-spawned one
        if (atomicSub(&spawner.spawn, 1) > 0) {
            // Update PRNG seed
            seed = pcg_hash(index ^ spawner.seed);

            // Initialize new particle
            var posVel = init_pos_vel(appear_area_index);
  
            vPos = (posVel.pos + appear_area_pos + spawner.origin);

            vVel = posVel.vel;
            vAge = 0.0;
            vLifetime = init_lifetime();
        } else {
            // Nothing to spawn; simply return without writing any update
            return;
        }
    }

{{FORCE_FIELD_CODE}}

    // Increment alive particle count and write indirection index
    let indirect_index = atomicAdd(&spawner.count, 1);
    indirect_buffer.indices[indirect_index] = index;

    // Write back particle itself
    particle_buffer.particles[index].pos = vPos;
    particle_buffer.particles[index].vel = vVel;
    particle_buffer.particles[index].age = vAge;
    particle_buffer.particles[index].lifetime = vLifetime;
}
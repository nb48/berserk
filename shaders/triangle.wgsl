@group(0) @binding(0)
var<uniform> uniforms: vec4<f32>; // x: vertical_offset, y: rotation, z: explosion_timer

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color_type: f32, // 0=triangle, 1=floor, 2=explosion
}

// 2D rotation function
fn rotate2D(pos: vec2<f32>, angle: f32) -> vec2<f32> {
    let cos_angle = cos(angle);
    let sin_angle = sin(angle);
    return vec2<f32>(
        pos.x * cos_angle - pos.y * sin_angle,
        pos.x * sin_angle + pos.y * cos_angle
    );
}

@vertex
fn vs_main(@builtin(vertex_index) vi : u32) -> VertexOutput {
    var output: VertexOutput;

    // Triangle vertices (indices 0-2)
    var triangle_pos = array<vec2<f32>, 3>(
        vec2<f32>( 0.0,  0.5),
        vec2<f32>(-0.5, -0.5),
        vec2<f32>( 0.5, -0.5)
    );

    // Floor vertices (indices 3-8) - two triangles making a rectangle
    var floor_pos = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -0.8),  // Bottom left
        vec2<f32>( 1.0, -0.8),  // Bottom right
        vec2<f32>(-1.0, -0.7),  // Top left
        vec2<f32>(-1.0, -0.7),  // Top left
        vec2<f32>( 1.0, -0.8),  // Bottom right
        vec2<f32>( 1.0, -0.7)   // Top right
    );

    if (vi < 3u) {
        // Triangle - apply rotation and offset
        var pos = triangle_pos[vi];

        // Apply rotation around the center of the triangle
        pos = rotate2D(pos, uniforms.y);

        // Apply vertical offset
        output.position = vec4<f32>(pos.x, pos.y + uniforms.x, 0.0, 1.0);
        output.color_type = 0.0;
    } else if (vi < 9u) {
        // Floor - no offset or rotation
        let floor_idx = vi - 3u;
        output.position = vec4<f32>(floor_pos[floor_idx].x, floor_pos[floor_idx].y, 0.0, 1.0);
        output.color_type = 1.0;
    } else {
        // Explosion particles (indices 9-38)
        let particle_idx = vi - 9u;
        let particle_id = particle_idx / 3u; // Which particle (0-9)
        let vertex_in_triangle = particle_idx % 3u; // Which vertex in triangle (0-2)

        // Generate particle based on ID
        let angle = f32(particle_id) * 0.628318; // 2*PI / 10 particles
        let explosion_progress = 1.0 - uniforms.z; // 0 to 1 as timer counts down

        // Particle flies outward and fades
        let speed = 0.8;
        let offset_x = cos(angle) * explosion_progress * speed;
        let offset_y = sin(angle) * explosion_progress * speed;

        // Triangle size decreases over time
        let size = 0.08 * uniforms.z;

        // Small triangle vertices relative to particle center
        var local_pos: vec2<f32>;
        if (vertex_in_triangle == 0u) {
            local_pos = vec2<f32>(0.0, size);
        } else if (vertex_in_triangle == 1u) {
            local_pos = vec2<f32>(-size * 0.866, -size * 0.5);
        } else {
            local_pos = vec2<f32>(size * 0.866, -size * 0.5);
        }

        // Position particle at center of where triangle was
        let center_x = 0.0;
        let center_y = uniforms.x;

        output.position = vec4<f32>(
            center_x + offset_x + local_pos.x,
            center_y + offset_y + local_pos.y,
            0.0,
            1.0
        );
        output.color_type = 2.0;
    }

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    if (input.color_type < 0.5) {
        // Triangle color - green (hide if explosion is active)
        if (uniforms.z > 0.0) {
            return vec4<f32>(0.0, 0.0, 0.0, 0.0); // Transparent during explosion
        }
        return vec4<f32>(0.2, 0.9, 0.4, 1.0);
    } else if (input.color_type < 1.5) {
        // Floor color - brown
        return vec4<f32>(0.6, 0.4, 0.2, 1.0);
    } else {
        // Explosion particle - only visible during explosion
        if (uniforms.z > 0.0) {
            // Color gradient: red to orange to yellow as they fade
            let fade = uniforms.z; // 1.0 to 0.0
            return vec4<f32>(1.0, 0.5 + fade * 0.5, 0.0, fade); // Orange/red with fade
        } else {
            return vec4<f32>(0.0, 0.0, 0.0, 0.0); // Transparent when no explosion
        }
    }
}
